use std::fs;
use std::io;
use std::io::ErrorKind;
use std::path::Path;

pub struct ShardMetadata {
    pub original_len: usize,
    pub new_len: usize,
}
pub struct SplittedShardData {
    pub shard_set: Vec<Vec<u8>>,
    pub shard_meta: ShardMetadata,
}

/// Reads an input file, zero-pads it so its length is divisible by `k`, and
/// splits the padded byte stream into `k` contiguous data shards of equal size.
///
/// The returned metadata preserves the original and padded lengths so later
/// stages can reconstruct the input and trim any trailing padding bytes after
/// decode.
///
/// Returns `InvalidInput` when:
/// - `k == 0`
/// - the input file is empty
/// - `k` is greater than the input byte length
pub fn split_into_shards(k: u8, input_filepath: &Path) -> Result<SplittedShardData, io::Error> {
    if k == 0 {
        return Err(io::Error::new(ErrorKind::InvalidInput, "k must be > 0"));
    }
    let k_1: usize = k as usize;

    // Read the full input into memory so it can be padded and chunked into shards.
    let mut bytes = fs::read(input_filepath)?;
    let original_len = bytes.len();
    if original_len == 0 {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            "input data byte-size must be > 0",
        ));
    }
    if k_1 > original_len {
        let err_msg = format!("k must be <= input data byte-size. current k={}", k_1);
        return Err(io::Error::new(ErrorKind::InvalidInput, err_msg));
    }

    // Track the padded length separately so reconstruction can remove trailing zeros.
    let mut new_len: usize = original_len;
    if original_len % k_1 != 0 {
        // Append zero bytes until the total length is evenly divisible by k.
        let extra_padded_len_req = k_1 - (original_len % k_1);
        new_len += extra_padded_len_req;
        bytes.resize(original_len + extra_padded_len_req, 0);
    }

    // Split the padded input into k contiguous shards of identical length.
    let shard_len = new_len / k_1;
    let shards: Vec<Vec<u8>> = bytes
        .chunks(shard_len)
        .map(|chunk| chunk.to_vec())
        .collect();
    let response = SplittedShardData {
        shard_set: shards,
        shard_meta: ShardMetadata {
            original_len,
            new_len,
        },
    };
    return Ok(response);
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::ErrorKind;

    fn remove_file_if_exists(path: &Path) {
        if path.exists() {
            fs::remove_file(path).unwrap();
        }
    }

    #[test]
    fn test_empty_input() {
        let input_path = std::env::temp_dir().join("splitter_test_empty_input.bin");
        remove_file_if_exists(&input_path);
        fs::write(&input_path, []).unwrap();

        let error = split_into_shards(2, &input_path).err().unwrap();

        assert_eq!(error.kind(), ErrorKind::InvalidInput);
        assert_eq!(error.to_string(), "input data byte-size must be > 0");

        fs::remove_file(&input_path).unwrap();
    }

    #[test]
    fn test_zero_k() {
        let input_path = Path::new("unused_for_zero_k.bin");

        let error = split_into_shards(0, input_path).err().unwrap();

        assert_eq!(error.kind(), ErrorKind::InvalidInput);
        assert_eq!(error.to_string(), "k must be > 0");
    }

    #[test]
    fn test_equal_divisions() {
        let input_path = std::env::temp_dir().join("splitter_test_equal_divisions.bin");
        let input_bytes = vec![1_u8, 2, 3, 4, 5, 6];
        remove_file_if_exists(&input_path);
        fs::write(&input_path, &input_bytes).unwrap();

        let result = split_into_shards(2, &input_path).unwrap();

        assert_eq!(result.shard_meta.original_len, 6);
        assert_eq!(result.shard_meta.new_len, 6);
        assert_eq!(result.shard_set.len(), 2);
        assert_eq!(result.shard_set[0], vec![1, 2, 3]);
        assert_eq!(result.shard_set[1], vec![4, 5, 6]);

        fs::remove_file(&input_path).unwrap();
    }

    #[test]
    fn test_unequal_divisions() {
        let input_path = std::env::temp_dir().join("splitter_test_unequal_divisions.bin");
        let input_bytes = vec![1_u8, 2, 3, 4, 5];
        remove_file_if_exists(&input_path);
        fs::write(&input_path, &input_bytes).unwrap();

        let result = split_into_shards(2, &input_path).unwrap();

        assert_eq!(result.shard_meta.original_len, 5);
        assert_eq!(result.shard_meta.new_len, 6);
        assert_eq!(result.shard_set.len(), 2);
        assert_eq!(result.shard_set[0], vec![1, 2, 3]);
        assert_eq!(result.shard_set[1], vec![4, 5, 0]);

        fs::remove_file(&input_path).unwrap();
    }

    #[test]
    fn k_greater_than_input_len() {
        let input_path = std::env::temp_dir().join("splitter_test_k_greater_than_input_len.bin");
        let input_bytes = vec![10_u8, 20, 30];
        remove_file_if_exists(&input_path);
        fs::write(&input_path, &input_bytes).unwrap();

        let error = split_into_shards(4, &input_path).err().unwrap();

        assert_eq!(error.kind(), ErrorKind::InvalidInput);
        assert_eq!(
            error.to_string(),
            "k must be <= input data byte-size. current k=4"
        );

        fs::remove_file(&input_path).unwrap();
    }

    #[test]
    fn k_equals_input_len() {
        let input_path = std::env::temp_dir().join("splitter_test_k_equals_input_len.bin");
        let input_bytes = vec![10_u8, 20, 30, 40];
        remove_file_if_exists(&input_path);

        fs::write(&input_path, &input_bytes).unwrap();

        let result = split_into_shards(4, &input_path).unwrap();

        assert_eq!(result.shard_meta.original_len, 4);
        assert_eq!(result.shard_meta.new_len, 4);
        assert_eq!(result.shard_set.len(), 4);
        assert_eq!(result.shard_set[0], vec![10]);
        assert_eq!(result.shard_set[1], vec![20]);
        assert_eq!(result.shard_set[2], vec![30]);
        assert_eq!(result.shard_set[3], vec![40]);

        fs::remove_file(&input_path).unwrap();
    }
}
