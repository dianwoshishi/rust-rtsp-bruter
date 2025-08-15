use std::vec::Vec;

// 凭据迭代器 - 用于生成用户名和密码的组合
#[derive(Clone)]
pub struct CredentialIterator {
    usernames: Vec<String>,
    passwords: Vec<String>,
    user_index: usize,
    pass_index: usize,
}

impl Iterator for CredentialIterator {
    type Item = (String, String);

    fn next(&mut self) -> Option<Self::Item> {
        if self.user_index >= self.usernames.len() {
            return None;
        }

        let current_user = self.usernames[self.user_index].clone();
        let current_pass = self.passwords[self.pass_index].clone();

        // 更新索引，准备下一次迭代
        self.pass_index += 1;
        if self.pass_index >= self.passwords.len() {
            self.pass_index = 0;
            self.user_index += 1;
        }

        Some((current_user, current_pass))
    }
}

impl CredentialIterator {
    pub fn new(usernames: Vec<String>, passwords: Vec<String>) -> Self {
        CredentialIterator {
            usernames,
            passwords,
            user_index: 0,
            pass_index: 0,
        }
    }
}
