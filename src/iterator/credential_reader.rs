use crate::iterator::credential_iterator::CredentialIterator;
use crate::errors::errors::RtspError;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::vec::Vec;

// 辅助函数：从文件读取非空行
fn read_lines_from_file(file_path: &str) -> Result<Vec<String>, RtspError> {
    let file = File::open(file_path).map_err(|e| RtspError::IoError(e))?;
    let reader = BufReader::new(file);
    let mut lines = Vec::new();

    for line in reader.lines() {
        let line = line.map_err(|e| RtspError::IoError(e))?;
        if !line.trim().is_empty() {
            lines.push(line.trim().to_string());
        }
    }

    Ok(lines)
}

// 定义凭据数据源类型
pub enum UsernameSource {
    FilePath(String),
    UsernameString(String),
}
pub enum PasswordSource {
    FilePath(String),
    PasswordString(String),
}

pub enum CredentialSource {
    UsernameAndPassword(UsernameSource, PasswordSource),
}

// 凭据读取器 - 支持多种凭据数据源
#[derive(Clone)]
pub struct CredentialReader<T> {
    source: T,
}

impl CredentialReader<CredentialSource> {
    // 从文件路径创建凭据读取器
    pub fn from_files(users_file: &str, passwords_file: &str) -> Self {
        CredentialReader {
            source: CredentialSource::UsernameAndPassword(
                UsernameSource::FilePath(users_file.to_string()),
                PasswordSource::FilePath(passwords_file.to_string()),
            ),
        }
    }

    // 从字符串创建凭据读取器
    pub fn from_strings(username: String, password: String) -> Self {
        CredentialReader {
            source: CredentialSource::UsernameAndPassword(
                UsernameSource::UsernameString(username),
                PasswordSource::PasswordString(password),
            ),
        }
    }

    // 从用户文件和密码字符串创建凭据读取器
    pub fn from_file_and_string(users_file: &str, password: String) -> Self {
        CredentialReader {
            source: CredentialSource::UsernameAndPassword(
                UsernameSource::FilePath(users_file.to_string()),
                PasswordSource::PasswordString(password),
            ),
        }
    }

    // 从用户字符串和密码文件创建凭据读取器
    pub fn from_string_and_file(username: String, passwords_file: &str) -> Self {
        CredentialReader {
            source: CredentialSource::UsernameAndPassword(
                UsernameSource::UsernameString(username),
                PasswordSource::FilePath(passwords_file.to_string()),
            ),
        }
    }

    // 读取用户名列表
    fn read_usernames(&self) -> Result<Vec<String>, RtspError> {
        match &self.source {
            CredentialSource::UsernameAndPassword(username_source, _) => match username_source {
                UsernameSource::FilePath(file_path) => read_lines_from_file(file_path),
                UsernameSource::UsernameString(username) => Ok(vec![username.clone()]),
            },
        }
    }

    // 读取密码列表
    fn read_passwords(&self) -> Result<Vec<String>, RtspError> {
        match &self.source {
            CredentialSource::UsernameAndPassword(_, password_source) => match password_source {
                PasswordSource::FilePath(file_path) => read_lines_from_file(file_path),
                PasswordSource::PasswordString(password) => Ok(vec![password.clone()]),
            },
        }
    }

    // 创建凭据迭代器
    pub fn into_iterator(&self) -> Result<CredentialIterator, RtspError> {
        let usernames = self.read_usernames()?;
        let passwords = self.read_passwords()?;

        Ok(CredentialIterator::new(usernames, passwords))
    }
}

// 为了向后兼容，保留原来的CredentialReader实现
pub type FileCredentialReader = CredentialReader<CredentialSource>;
