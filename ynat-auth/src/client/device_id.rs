use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

use crate::error::AuthError;

pub struct DeviceIdStore {
    device_id_path: PathBuf,
}

impl DeviceIdStore {
    pub fn new() -> Result<Self, AuthError> {
        let cache_dir = dirs::cache_dir()
            .ok_or_else(|| AuthError::Configuration("Could not find cache directory".to_string()))?
            .join("ynat");

        if !cache_dir.exists() {
            fs::create_dir_all(&cache_dir)?;
        }

        Ok(Self {
            device_id_path: cache_dir.join("device_id"),
        })
    }

    pub fn load_or_create(&self) -> Result<String, AuthError> {
        if self.device_id_path.exists() {
            // Load existing device ID
            Ok(fs::read_to_string(&self.device_id_path)?.trim().to_string())
        } else {
            // Generate new device ID
            let device_id = Uuid::new_v4().to_string();
            fs::write(&self.device_id_path, &device_id)?;

            // Set permissions to 0600 (owner read/write only)
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&self.device_id_path)?.permissions();
                perms.set_mode(0o600);
                fs::set_permissions(&self.device_id_path, perms)?;
            }

            Ok(device_id)
        }
    }
}
