use std::io::{self, ErrorKind::InvalidInput};

use crate::gf256_constants::GF256_INVERSE_TABLE;

pub type ByteMatrix = Vec<Vec<Byte>>;
pub type Byte = u8;

enum ElementaryRowOp {
    PivotNormalisation,
    RowElimination,
}

const ZERO_BYTE: Byte = 0b00;

/// The polynomial used for reducing a higher degree term to a lower degree term
/// in field multiplication over `GF(2^8)`
///
/// The reduction step uses `0x1d` instead of `0x11d` because the `x^8` term has
/// already been discarded by the left shift on an 8-bit value. XORing with
/// `0x1d` is therefore equivalent to subtracting the full modulus after
/// overflow, which is the usual optimized form for byte-sized finite-field
/// multiplication.
const REDUCING_POLYNOMIAL_MODULUS: Byte = 0x1d;

fn is_valid_matrix(mat: &ByteMatrix) -> Result<(usize, usize), io::Error> {
    let m: usize = mat.len();
    if m == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "zero rows in matrix",
        ));
    }
    let n: usize = mat[0].len();
    if n == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "zero columns in matrix",
        ));
    }
    for (row_idx, row) in mat.iter().enumerate() {
        if row.len() != n {
            let err_msg = format!(
                "non-rectangular matrix. row_0_col_cnt={} & row_{}_col_cnt={}",
                n,
                row_idx,
                row.len()
            );
            return Err(io::Error::new(io::ErrorKind::InvalidData, err_msg));
        }
    }
    return Ok((m, n));
}

pub fn is_square(mat: &ByteMatrix) -> Result<usize, io::Error> {
    let (m, n) = is_valid_matrix(mat)?;
    if m == n {
        return Ok(m);
    }
    let err_msg = format!("matrix is not square. m={}, n={}", m, n);
    return Err(io::Error::new(io::ErrorKind::InvalidData, err_msg));
}

pub fn is_rectangular(mat: &ByteMatrix) -> Result<bool, io::Error> {
    let (m, n) = is_valid_matrix(mat)?;
    if m == n {
        return Ok(false);
    }
    return Ok(true);
}

pub fn get_dimensions(mat: &ByteMatrix) -> (usize, usize) {
    let m: usize = mat.len();
    let n: usize = mat[0].len();
    return (m, n);
}

pub fn row_count(mat: &ByteMatrix) -> usize {
    let m: usize = mat.len();
    return m;
}

pub fn col_count(mat: &ByteMatrix) -> usize {
    let n: usize = mat[0].len();
    return n;
}

pub fn get_row(mat: &ByteMatrix, idx: usize) -> Result<&Vec<Byte>, io::Error> {
    let (m, _) = is_valid_matrix(mat)?;
    if idx >= m {
        let err_msg = format!("index out of bounds. row_cnt={} & input_idx={}", m, idx);
        return Err(io::Error::new(io::ErrorKind::InvalidInput, err_msg));
    }
    return Ok(&mat[idx]);
}

pub fn get_col(mat: &ByteMatrix, idx: usize) -> Result<Vec<Byte>, io::Error> {
    let (m, n) = is_valid_matrix(mat)?;
    if idx >= n {
        let err_msg = format!("index out of bounds. col_cnt={} & input_idx={}", n, idx);
        return Err(io::Error::new(io::ErrorKind::InvalidInput, err_msg));
    }

    let mut col: Vec<Byte> = vec![];
    for i in 0..m {
        col.push(mat[i][idx].clone());
    }
    return Ok(col);
}

pub fn build_identity_matrix(size: usize) -> Result<ByteMatrix, io::Error> {
    if size <= 0 {
        let err_msg = format!("cannot create matrix for size={}", size);
        return Err(io::Error::new(io::ErrorKind::InvalidInput, err_msg));
    }
    let mut mat: Vec<Vec<Byte>> = vec![];
    for i in 0..size {
        let mut row: Vec<u8> = vec![0; size];
        row[i] = 1;
        mat.push(row);
    }
    return Ok(mat);
}

pub fn multiply_matrices(a: &ByteMatrix, b: &ByteMatrix) -> Result<ByteMatrix, io::Error> {
    let (r1, c1) = is_valid_matrix(a)?;
    let (r2, c2) = is_valid_matrix(b)?;
    if c1 != r2 {
        let err_msg = format!(
            "Multiplication Condition Not Satisfied. # of cols in Mat-1 ({}) != # of rows in Mat-2 ({})",
            c1, r2
        );
        return Err(io::Error::new(io::ErrorKind::InvalidInput, err_msg));
    }
    let n = c1;
    let mut product_mat: ByteMatrix = vec![vec![ZERO_BYTE; c2]; r1];
    for i in 0..r1 {
        for j in 0..c2 {
            let row_i = get_row(a, i)?;
            let mut p_ij: Byte = ZERO_BYTE;
            for x in 0..n {
                p_ij = p_ij ^ field_multiply(row_i[x], b[x][j]); // field addition
            }
            product_mat[i][j] = p_ij;
        }
    }
    return Ok(product_mat);
}

/// Multiplies two field elements in `GF(2^8)`.
///
/// Each input byte is interpreted as a polynomial over `GF(2)`, where the bit
/// at position `i` is the coefficient of `x^i`. For example, `0b0000_1011`
/// represents `x^3 + x + 1`.
///
/// Arithmetic in this field differs from ordinary integer arithmetic:
///
/// - Addition is bitwise XOR, because coefficients are in `GF(2)`.
/// - Multiplication is polynomial multiplication followed by reduction modulo
///   the irreducible polynomial `x^8 + x^4 + x^3 + x^2 + 1` (`0x11d`).
///
/// This implementation uses the standard "shift-and-add" approach:
///
/// - Walk through the bits of `b` from least-significant to most-significant.
/// - Whenever the current bit of `b` is `1`, XOR the current value of `a` into
///   the result.
/// - After each step, multiply `a` by `x` by shifting left one bit.
/// - If that shift would create an `x^8` term, immediately reduce by XORing the
///   low 8 bits of the modulus, `0x1d`.
fn field_multiply(mut a: Byte, mut b: Byte) -> Byte {
    let mut res: Byte = ZERO_BYTE;
    while b > 0 {
        // current coefficent of b == 1
        // include the current multiple of a
        if lsb(b) == 1 {
            res = res ^ a;
        }

        let msb_of_a = msb(a);

        // increase degree of a by 1 (equivalent to multiplying by x)
        a = a << 1;

        // we got a term with degree >= 8 => reduce it using REDUCING_POLYNOMIAL_MODULUS
        if msb_of_a == 1 {
            a = a ^ REDUCING_POLYNOMIAL_MODULUS;
        }

        // examine the next coefficient of b
        b = b >> 1;
    }
    return res;
}

fn lsb(b: Byte) -> Byte {
    return b & 0x01;
}

fn msb(a: Byte) -> Byte {
    return (a & (0x01 << 7)) >> 7;
}

/// Computes the inverse of a square matrix over `GF(2^8)`.
///
/// Each byte is treated as a field element, not as an ordinary integer. This
/// uses Gauss-Jordan elimination on the augmented matrix `[mat | I]`, with
/// field multiplication for row scaling and XOR-based subtraction for row
/// elimination. The returned matrix satisfies `mat * inverse = I`.
///
/// # Errors
///
/// Returns an error when `mat` is invalid or non-square, or when it is
/// singular and therefore has no inverse.
pub fn invert_matrix(mat: &ByteMatrix) -> Result<ByteMatrix, io::Error> {
    let n: usize = is_square(mat)?;
    let mut a = mat.clone();
    let mut res = build_identity_matrix(n)?;
    for i in 0..n {
        // swap with a row with non-zero pivot
        if a[i][i] == 0 {
            swap_rows(&mut a, &mut res, n, i)?;
        }
        // normalise the pivot
        // multiply each element of row-i with inverse(a[i][i])
        let mult_factor = field_inverse(a[i][i])?;
        for c in 0..n {
            a[i][c] = field_multiply(mult_factor, a[i][c]);
            res[i][c] = field_multiply(mult_factor, res[i][c]);
        }

        // make all non-pivot elements in the column-i = 0
        for r in 0..n {
            if r == i {
                continue;
            }
            let mult_factor = a[r][i];
            for c in 0..n {
                a[r][c] = field_subtract(a[r][c], field_multiply(mult_factor, a[i][c]));
                res[r][c] = field_subtract(res[r][c], field_multiply(mult_factor, res[i][c]));
            }
        }
    }
    Ok(res)
}

fn swap_rows(
    a: &mut ByteMatrix,
    res: &mut ByteMatrix,
    n: usize,
    i: usize,
) -> Result<(), io::Error> {
    let mut swap_done = false;
    for r in (i + 1)..n {
        if a[r][i] != 0b00 {
            let mut tmp: Vec<Byte> = a[i].clone();
            a[i] = a[r].clone();
            a[r] = tmp;

            tmp = res[i].clone();
            res[i] = res[r].clone();
            res[r] = tmp;

            swap_done = true;
            break;
        }
    }
    if !swap_done {
        return Err(io::Error::new(InvalidInput, "Matrix is singular"));
    }
    Ok(())
}

/// Returns the multiplicative inverse of a nonzero `GF(2^8)` field element.
///
/// The inverse is looked up from `GF256_INVERSE_TABLE`, whose values use the
/// same reducing polynomial as [`field_multiply`]. For a nonzero `a`, the
/// returned value satisfies `field_multiply(a, inverse) == 1`.
///
/// # Errors
///
/// Returns an error for zero because it has no multiplicative inverse.
fn field_inverse(a: Byte) -> Result<Byte, io::Error> {
    if a == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "zero has no multiplicative inverse",
        ));
    }
    return Ok(GF256_INVERSE_TABLE[a as usize]);
}

fn field_subtract(a: Byte, b: Byte) -> Byte {
    // In GF(2^8), each bit is a polynomial coefficient in GF(2).
    // Since arithmetic is modulo 2
    // each element is its own additive inverse
    // so b = -b
    return a ^ b;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reference_field_multiply(a: Byte, b: Byte) -> Byte {
        let mut product: u16 = 0;
        for bit in 0..8 {
            if ((b >> bit) & 1) == 1 {
                product ^= (a as u16) << bit;
            }
        }

        let reducing_polynomial: u16 = 0x11d;
        for bit in (8..=14).rev() {
            if ((product >> bit) & 1) == 1 {
                product ^= reducing_polynomial << (bit - 8);
            }
        }

        product as Byte
    }

    fn reference_matrix_multiply(a: &ByteMatrix, b: &ByteMatrix) -> ByteMatrix {
        let rows = a.len();
        let cols = b[0].len();
        let inner = a[0].len();
        let mut result = vec![vec![ZERO_BYTE; cols]; rows];

        for i in 0..rows {
            for j in 0..cols {
                let mut acc = ZERO_BYTE;
                for k in 0..inner {
                    acc ^= reference_field_multiply(a[i][k], b[k][j]);
                }
                result[i][j] = acc;
            }
        }

        result
    }

    #[test]
    fn field_multiply_matches_reference_cases() {
        let cases = [
            (0x00, 0x00),
            (0x00, 0x53),
            (0x01, 0xae),
            (0x02, 0x80),
            (0x53, 0xca),
            (0xff, 0x13),
            (0x87, 0x87),
        ];

        for (a, b) in cases {
            assert_eq!(field_multiply(a, b), reference_field_multiply(a, b));
            assert_eq!(field_multiply(b, a), reference_field_multiply(b, a));
        }
    }

    #[test]
    fn field_multiply_distributes_over_xor() {
        let a = 0x57;
        let b = 0x83;
        let c = 0x13;

        assert_eq!(
            field_multiply(a, b ^ c),
            field_multiply(a, b) ^ field_multiply(a, c)
        );
    }

    #[test]
    fn multiply_matrices_computes_gf_256_product() {
        let a: ByteMatrix = vec![vec![0x01, 0x02, 0x03], vec![0x57, 0x83, 0x1d]];
        let b: ByteMatrix = vec![vec![0x02, 0x05], vec![0x07, 0x0b], vec![0x0d, 0x11]];

        let product = multiply_matrices(&a, &b).unwrap();

        assert_eq!(product, reference_matrix_multiply(&a, &b));
    }

    #[test]
    fn multiply_matrices_identity_preserves_matrix() {
        let matrix: ByteMatrix = vec![vec![0x01, 0x02], vec![0x53, 0xca], vec![0xff, 0x00]];
        let identity = build_identity_matrix(3).unwrap();

        let product = multiply_matrices(&identity, &matrix).unwrap();

        assert_eq!(product, matrix);
    }

    #[test]
    fn multiply_matrices_rejects_dimension_mismatch() {
        let a: ByteMatrix = vec![vec![0x01, 0x02]];
        let b: ByteMatrix = vec![vec![0x03, 0x04]];

        let err = multiply_matrices(&a, &b).unwrap_err();

        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
        assert!(
            err.to_string()
                .contains("Multiplication Condition Not Satisfied")
        );
    }

    #[test]
    fn inverse_table_matches_field_multiplication() {
        assert_eq!(GF256_INVERSE_TABLE[0], 0x00);

        for value in 1..=u8::MAX {
            let inverse = GF256_INVERSE_TABLE[value as usize];
            assert_ne!(inverse, 0x00, "missing inverse for 0x{value:02x}");
            assert_eq!(
                field_multiply(value, inverse),
                0x01,
                "incorrect inverse for 0x{value:02x}"
            );
        }
    }

    #[test]
    fn invert_matrix_returns_a_two_sided_inverse() {
        let matrix: ByteMatrix = vec![vec![0x02, 0x01], vec![0x01, 0x01]];
        let inverse = invert_matrix(&matrix).unwrap();
        let identity = build_identity_matrix(2).unwrap();

        assert_eq!(multiply_matrices(&matrix, &inverse).unwrap(), identity);
        assert_eq!(multiply_matrices(&inverse, &matrix).unwrap(), identity);
    }

    #[test]
    fn invert_matrix_swaps_rows_when_the_pivot_is_zero() {
        let matrix: ByteMatrix = vec![vec![0x00, 0x01], vec![0x01, 0x00]];

        let inverse = invert_matrix(&matrix).unwrap();

        assert_eq!(inverse, matrix);
    }

    #[test]
    fn invert_matrix_rejects_a_singular_matrix() {
        let singular: ByteMatrix = vec![vec![0x01, 0x01], vec![0x01, 0x01]];

        let err = invert_matrix(&singular).unwrap_err();

        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
        assert!(err.to_string().contains("Matrix is singular"));
    }

    #[test]
    fn invert_matrix_rejects_a_non_square_matrix() {
        let non_square: ByteMatrix = vec![vec![0x01, 0x02, 0x03], vec![0x04, 0x05, 0x06]];

        let err = invert_matrix(&non_square).unwrap_err();

        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert!(err.to_string().contains("matrix is not square"));
    }
}
