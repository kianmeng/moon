use cached::proc_macro::cached;
use moon_error::MoonError;
use moon_lang::LockfileDependencyVersions;
use moon_utils::fs;
use std::collections::HashMap;
use std::path::PathBuf;
use yarn_lock_parser::{parse_str, Entry};

#[cached(result)]
pub async fn load_lockfile_dependencies(
    path: PathBuf,
) -> Result<LockfileDependencyVersions, MoonError> {
    let mut deps: LockfileDependencyVersions = HashMap::new();

    let yarn_lock_text = fs::read(&path).await?;
    let entries: Vec<Entry> = parse_str(&yarn_lock_text)
        .map_err(|e| MoonError::Generic(format!("Failed to parse lockfile: {}", e)))?;

    for entry in entries {
        if entry.integrity.is_empty() {
            // all workspace dependencies have empty integrities, so we will skip them
            continue;
        }
        let dep = deps.entry(entry.name.to_owned()).or_default();
        dep.push(entry.integrity.to_owned());
    }

    Ok(deps)
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;
    use moon_utils::string_vec;
    use pretty_assertions::assert_eq;

    #[tokio::test]
    async fn parses_lockfile() {
        let temp = assert_fs::TempDir::new().unwrap();

        temp.child("yarn.lock").write_str(r#"
# This file is generated by running "yarn install" inside your project.
# Manual changes might be lost - proceed with caution!

__metadata:
  version: 6
  cacheKey: 8

"is-buffer@npm:^1.1.5":
  version: 1.1.6
  resolution: "is-buffer@npm:1.1.6"
  checksum: 4a186d995d8bbf9153b4bd9ff9fd04ae75068fe695d29025d25e592d9488911eeece84eefbd8fa41b8ddcc0711058a71d4c466dcf6f1f6e1d83830052d8ca707
  languageName: node
  linkType: hard

"is-even@npm:^1.0.0":
  version: 1.0.0
  resolution: "is-even@npm:1.0.0"
  dependencies:
    is-odd: ^0.1.2
  checksum: 0267545d7cb6724aee249e88942cf22f6263aa006cd9bf83c2ddbb2a1d47280e8c4d72b2d50e38bd3575df717c993904b44153cc1772a55dabca250ca40cc4f7
  languageName: node
  linkType: hard

"is-number@npm:^3.0.0":
  version: 3.0.0
  resolution: "is-number@npm:3.0.0"
  dependencies:
    kind-of: ^3.0.2
  checksum: 0c62bf8e9d72c4dd203a74d8cfc751c746e75513380fef420cda8237e619a988ee43e678ddb23c87ac24d91ac0fe9f22e4ffb1301a50310c697e9d73ca3994e9
  languageName: node
  linkType: hard

"is-odd@npm:^0.1.2":
  version: 0.1.2
  resolution: "is-odd@npm:0.1.2"
  dependencies:
    is-number: ^3.0.0
  checksum: 146069d7622c991c75c17ca63bccf5470cd730c24082874e53e797a10ff38a896197d6ce34ad137a2f422dcc614b10ff24d31fe93dcdb29f0cb758f2d924f477
  languageName: node
  linkType: hard

"kind-of@npm:^3.0.2":
  version: 3.2.2
  resolution: "kind-of@npm:3.2.2"
  dependencies:
    is-buffer: ^1.1.5
  checksum: e898df8ca2f31038f27d24f0b8080da7be274f986bc6ed176f37c77c454d76627619e1681f6f9d2e8d2fd7557a18ecc419a6bb54e422abcbb8da8f1a75e4b386
  languageName: node
  linkType: hard

"test@workspace:.":
  version: 0.0.0-use.local
  resolution: "test@workspace:."
  dependencies:
    is-even: ^1.0.0
  languageName: unknown
  linkType: soft
"#.trim()).unwrap();

        assert_eq!(
            load_lockfile_dependencies(temp.path().join("yarn.lock")).await.unwrap(),
            HashMap::from([
                (
                    "is-buffer".to_owned(),
                    string_vec!["4a186d995d8bbf9153b4bd9ff9fd04ae75068fe695d29025d25e592d9488911eeece84eefbd8fa41b8ddcc0711058a71d4c466dcf6f1f6e1d83830052d8ca707"]
                ),
                (
                    "is-odd".to_owned(),
                    string_vec!["146069d7622c991c75c17ca63bccf5470cd730c24082874e53e797a10ff38a896197d6ce34ad137a2f422dcc614b10ff24d31fe93dcdb29f0cb758f2d924f477"]
                ),
                ("kind-of".to_owned(), string_vec!["e898df8ca2f31038f27d24f0b8080da7be274f986bc6ed176f37c77c454d76627619e1681f6f9d2e8d2fd7557a18ecc419a6bb54e422abcbb8da8f1a75e4b386"]),
                ("is-even".to_owned(), string_vec!["0267545d7cb6724aee249e88942cf22f6263aa006cd9bf83c2ddbb2a1d47280e8c4d72b2d50e38bd3575df717c993904b44153cc1772a55dabca250ca40cc4f7"]),
                ("is-number".to_owned(), string_vec!["0c62bf8e9d72c4dd203a74d8cfc751c746e75513380fef420cda8237e619a988ee43e678ddb23c87ac24d91ac0fe9f22e4ffb1301a50310c697e9d73ca3994e9"]),
            ])
        );

        temp.close().unwrap();
    }
}
