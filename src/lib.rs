use std::{
    fs::{metadata, read_dir, remove_dir_all},
    path::Path,
};

use zed_extension_api::{
    self as zed, DownloadedFileType, Extension, GithubReleaseOptions, LanguageServerId,
    LanguageServerInstallationStatus, Os, Worktree, current_platform, download_file,
    latest_github_release, make_file_executable, register_extension, serde_json::Value,
    set_language_server_installation_status, settings::LspSettings,
};

struct Sql {
    command_path: Option<String>,
}

impl Sql {
    fn get_command_path(&mut self, language_server_id: &LanguageServerId) -> zed::Result<String> {
        if let Some(command_path) = &self.command_path {
            if metadata(command_path).is_ok_and(|metadata| metadata.is_file()) {
                return Ok(command_path.clone());
            }
        }

        set_language_server_installation_status(
            language_server_id,
            &LanguageServerInstallationStatus::CheckingForUpdate,
        );

        let latest_release = latest_github_release(
            "sqls-server/sqls",
            GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;
        let ls_path = format!("{language_server_id}/");
        let version_path = format!("{ls_path}{}/", latest_release.version);
        let mut command_path = format!("{version_path}sqls");

        if current_platform().0 == Os::Windows {
            command_path += ".exe";
        }

        if !metadata(&command_path).is_ok_and(|metadata| metadata.is_file()) {
            let asset_name = format!(
                "sqls-{}-{}.zip",
                match current_platform().0 {
                    Os::Mac => "darwin",
                    Os::Linux => "linux",
                    Os::Windows => "windows",
                },
                // Substring needed to remove the leading 'v'
                &latest_release.version[1..],
            );
            let asset = latest_release
                .assets
                .into_iter()
                .find(|asset| asset.name == asset_name)
                .ok_or("no asset found for platform")?;

            set_language_server_installation_status(
                language_server_id,
                &LanguageServerInstallationStatus::Downloading,
            );
            download_file(&asset.download_url, &version_path, DownloadedFileType::Zip)?;
            make_file_executable(&command_path)?;

            for entry in read_dir(ls_path).map_err(|err| err.to_string())? {
                let entry = entry.map_err(|err| err.to_string())?;
                let entry_path = entry.path();

                if entry_path != Path::new(&version_path) {
                    remove_dir_all(entry_path).map_err(|err| err.to_string())?;
                }
            }
        }

        self.command_path = Some(command_path.clone());

        set_language_server_installation_status(
            language_server_id,
            &LanguageServerInstallationStatus::None,
        );

        Ok(command_path)
    }
}

impl Extension for Sql {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self { command_path: None }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        _worktree: &Worktree,
    ) -> zed::Result<zed::Command> {
        Ok(zed::Command {
            command: self.get_command_path(language_server_id)?,
            args: Vec::new(),
            env: Vec::new(),
        })
    }

    fn language_server_workspace_configuration(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> zed_extension_api::Result<Option<Value>> {
        LspSettings::for_worktree(language_server_id.as_ref(), worktree)
            .map(|settings| settings.settings)
    }
}

register_extension!(Sql);
