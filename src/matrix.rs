use std::io;

pub type ByteMatrix = Vec<Vec<u8>>;

pub fn is_square(mat: &ByteMatrix) -> Result<(), io::Error> {
    Ok(())
}

pub fn is_rectangular(mat: &ByteMatrix) -> Result<(), io::Error> {
    Ok(())
}

pub fn get_dimensions(mat: &ByteMatrix) -> Result<(usize, usize), io::Error> {
    return Ok((0, 0));
}

pub fn row_count(mat: &ByteMatrix) -> usize {
    return 0;
}

pub fn col_count(mat: &ByteMatrix) -> usize {
    return 0;
}

pub fn get_row(mat: &ByteMatrix, idx: usize) -> Result<Vec<u8>, io::Error> {
    Ok(())
}

pub fn get_col(mat: &ByteMatrix, idx: usize) -> Result<Vec<u8>, io::Error> {
    Ok(())
}

pub fn build_identity_matrix(size: usize) -> Result<ByteMatrix, io::Error> {
    Ok(())
}

pub fn multiply_matrices(a: &ByteMatrix, b: &ByteMatrix) -> Result<ByteMatrix, io::Error> {
    Ok(())
}

pub fn invert_matrix(mat: &ByteMatrix) -> Result<ByteMatrix, io::Error> {
    Ok(())
}
