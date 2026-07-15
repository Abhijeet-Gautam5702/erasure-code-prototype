use std::io::{self, ErrorKind::InvalidData};

pub type ByteMatrix = Vec<Vec<Byte>>;
pub type Byte = u8;

const ZERO_BYTE: u8 = 0u8;

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

pub fn is_square(mat: &ByteMatrix) -> Result<bool, io::Error> {
    let (m, n) = is_valid_matrix(mat)?;
    if m == n {
        return Ok(true);
    }
    return Ok(false);
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
        let err_msg = format!("# of cols in Mat-1 ({}) != # of rows in Mat-2 ({})", c1, r2);
        return Err(io::Error::new(io::ErrorKind::InvalidInput, err_msg));
    }
    let n = c1;
    let mut product_mat: ByteMatrix = vec![vec![ZERO_BYTE; c2]; r1];
    for i in 0..r1 {
        for j in 0..c2 {
            let row_i = get_row(a, i)?;
            let col_j = get_col(b, j)?;
            let mut p_ij: Byte = ZERO_BYTE;
            for x in 0..n {
                p_ij = p_ij ^ field_multiply(&row_i[x], &col_j[x]);
            }
            product_mat[i][j] = p_ij;
        }
    }
    return Ok(product_mat);
}

fn field_multiply(a: &Byte, b: &Byte) -> Byte {
    Ok(vec![])
}

pub fn invert_matrix(mat: &ByteMatrix) -> Result<ByteMatrix, io::Error> {
    Ok(())
}
