use crate::writer;
use openssl::sha::Sha1;

/// Hashes a blob by first using the writer struct to attach the necessary header to
/// the blob, and then hash it using sha1, and returns the hash.
pub fn hash_blob(blob: &String) -> [u8; 20] {
    let mut hasher = Sha1::new();
    let blob_with_header = writer::prepend_header_to_blob(blob);
    hasher.update(&blob_with_header);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex::ToHex;
    use std::string::ToString;

    #[test]
    fn hasher_correctly_hashes_blob() {
        let blob = "tree TreeTest\n\
                    parent ParentTest\n\
                    author AuthorTest <test@test.com> 1454691142 -0000\n\
                    committer AuthorTest <test@test.com> 1454691142 -0000\n\n\
                    MessageTest\n"
            .to_string();
        let hash = hash_blob(&blob);
        assert_eq!(
            &hash.encode_hex::<String>(),
            "651478bce64904caf84aaa364d8d7dbee0698f54"
        );
    }

    #[test]
    fn clone_base_hasher_works() {
        let blob = "tree TreeTest\n\
                    parent ParentTest\n\
                    author AuthorTest <test@test.com> 1454691142 -0000\n\
                    committer AuthorTest <test@test.com> 1454691142 -0000\n\n\
                    MessageTest\n"
            .to_string();

        let base_hasher = base_hasher(&blob);
        let hasher1 = base_hasher.clone();
        let hash1 = update_hash_with_nonce(hasher1, 20);

        let hasher2 = base_hasher.clone();
        let hash2 = update_hash_with_nonce(hasher2, 20);
        assert_eq!(
            &hash1.encode_hex::<String>(),
            "9190ffb25997d4ae904050de890129c8fd2f77e3"
        );
        assert_eq!(
            &hash2.encode_hex::<String>(),
            "9190ffb25997d4ae904050de890129c8fd2f77e3"
        );
    }
}

