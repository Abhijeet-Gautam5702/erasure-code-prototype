use std::io;

pub type ByteMatrix = Vec<Vec<u8>>;

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

pub fn get_row(mat: &ByteMatrix, idx: usize) -> Result<&Vec<u8>, io::Error> {
    let (m, _) = is_valid_matrix(mat)?;
    if idx >= m {
        let err_msg = format!("index out of bounds. row_cnt={} & input_idx={}", m, idx);
        return Err(io::Error::new(io::ErrorKind::InvalidInput, err_msg));
    }
    return Ok(&mat[idx]);
}

pub fn get_col(mat: &ByteMatrix, idx: usize) -> Result<Vec<u8>, io::Error> {
    let (m, n) = is_valid_matrix(mat)?;
    if idx >= n {
        let err_msg = format!("index out of bounds. col_cnt={} & input_idx={}", n, idx);
        return Err(io::Error::new(io::ErrorKind::InvalidInput, err_msg));
    }

    let mut col: Vec<u8> = vec![];
    for i in 0..m {
        col.push(mat[i][idx]);
    }
    return Ok(col);
}

pub fn build_identity_matrix(size: usize) -> Result<ByteMatrix, io::Error> {
    if size <= 0 {
        let err_msg = format!("cannot create matrix for size={}", size);
        return Err(io::Error::new(io::ErrorKind::InvalidInput, err_msg));
    }
    let mut mat: Vec<Vec<u8>> = vec![];
    for i in 0..size {
        let mut row: Vec<u8> = vec![0; size];
        row[i] = 1;
        mat.push(row);
    }
    return Ok(mat);
}

pub fn multiply_matrices(a: &ByteMatrix, b: &ByteMatrix) -> Result<ByteMatrix, io::Error> {
    Ok(())
}

pub fn invert_matrix(mat: &ByteMatrix) -> Result<ByteMatrix, io::Error> {
    Ok(())
}
