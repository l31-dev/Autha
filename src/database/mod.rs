pub mod cassandra;
pub mod mem;

use crate::helpers::crypto;
use anyhow::Result;

/// Tries to find a user in cache or use Cassandra database
pub fn get_user(vanity: String, requester: String) -> Result<crate::model::user::User> {
    match mem::get(vanity.clone())? {
        Some(data) => {
            Ok(serde_json::from_str(&data[..])?)
        },
        None => {
            let mut cassandra = cassandra::query("SELECT username, avatar, bio, deleted, flags, email, birthdate, verified FROM accounts.users WHERE vanity = ?", vec![vanity.clone()])?.get_body()?.as_cols().unwrap().rows_content.clone();

            if cassandra.is_empty() {
                cassandra = cassandra::query("SELECT username, avatar, bio, deleted, flags, email, birthdate FROM accounts.bots WHERE id = ?", vec![vanity.clone()])?.get_body()?.as_cols().unwrap().rows_content.clone();
            }

            println!("{:?}", cassandra);

            if cassandra.is_empty() {
                Ok(crate::model::user::User {
                    username: "".to_string(),
                    vanity:  "".to_string(),
                    avatar: None,
                    bio: None,
                    email: None,
                    birthdate: None,
                    deleted: false,
                    flags: 0,
                    verified: false,
                })
            } else {
                Ok(crate::model::user::User {
                    username: std::str::from_utf8(&cassandra[0][0].clone().into_plain().unwrap()[..]).unwrap().to_string(),
                    avatar: if cassandra[0][1].clone().into_plain().is_none() { None } else { let res = std::str::from_utf8(&cassandra[0][1].clone().into_plain().unwrap()[..]).unwrap_or("").to_string(); if res.is_empty() { None } else { Some(res) } },
                    bio: if cassandra[0][2].clone().into_plain().is_none() { None } else { let res = std::str::from_utf8(&cassandra[0][2].clone().into_plain().unwrap()[..]).unwrap_or("").to_string(); if res.is_empty() { None } else { Some(res) } },
                    email: if vanity == requester { Some(crypto::fpe_decrypt(std::str::from_utf8(&cassandra[0][5].clone().into_plain().unwrap()[..])?.to_string())?) } else { None },
                    birthdate: if cassandra[0][6].clone().into_plain().is_none() { None } else { let res = std::str::from_utf8(&cassandra[0][6].clone().into_plain().unwrap()[..])?.to_string(); if res.is_empty() { None } else { Some(crypto::fpe_decrypt(res)?) } },
                    deleted: cassandra[0][3].clone().into_plain().unwrap()[..] != [0],
                    flags: u32::from_be_bytes((&cassandra[0][4].clone().into_plain().unwrap()[..])[..4].try_into().unwrap()),
                    verified: false,
                    vanity,
                })
            }
        }
    }
}
