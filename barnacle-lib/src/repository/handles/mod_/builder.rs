use std::{
    fs::{File, create_dir_all},
    path::{Path, PathBuf},
};

use compress_tools::{Ownership, list_archive_files, uncompress_archive};
use fs_more::directory::{
    DirectoryCopyOptions, DirectoryMoveOptions, SymlinkBehaviour, copy_directory, move_directory,
};
use sea_orm::{ActiveValue::Set, EntityTrait};
use tempfile::tempdir;

use crate::{
    Game, Mod,
    repository::{
        config::Cfg,
        db::{
            Db,
            models::mods::{ActiveModel, Entity},
        },
    },
};

#[must_use]
pub struct ModBuilder {
    db: Db,
    cfg: Cfg,
    game: Game,
    name: String,
}

impl ModBuilder {
    pub(crate) fn new(db: &Db, cfg: &Cfg, game: &Game, name: &str) -> Self {
        Self {
            db: db.clone(),
            cfg: cfg.clone(),
            game: game.clone(),
            name: name.to_string(),
        }
    }

    async fn add(&self) -> Mod {
        add_mod(&self.db, &self.cfg, &self.game, &self.name).await
    }

    /// Create a new [`Mod`] with no contents
    pub async fn create(&self) -> Mod {
        self.add().await
    }

    /// Create a new [`Mod`], copying the contents from the given path
    pub async fn from_dir(&self, path: &Path) -> Mod {
        if !path.is_dir() {
            panic!("Not a directory");
        }

        // TODO: Wrap in transaction
        let mod_ = self.add().await;

        let dest = mod_.dir().await.unwrap();

        // TODO: Use copy_directory_with_progress so we can, you know, report progress.
        copy_directory(
            path,
            dest,
            DirectoryCopyOptions {
                symlink_behaviour: SymlinkBehaviour::Follow,
                ..Default::default()
            },
        )
        .unwrap();

        mod_
    }

    pub async fn from_archive(&self, name: &str, path: &Path) -> ModArchiveImport {
        ModArchiveImport::new(&self.db, &self.cfg, &self.game, name, path)
    }
}

#[must_use]
pub struct ModArchiveImport {
    db: Db,
    cfg: Cfg,
    game: Game,
    name: String,
    archive_path: PathBuf,
    entries: Vec<PathBuf>,
    /// A relative path to the directory that should be the root of the [`Mod`]
    root: Option<PathBuf>,
}

impl ModArchiveImport {
    fn new(db: &Db, cfg: &Cfg, game: &Game, name: &str, path: &Path) -> Self {
        if !path.is_file() {
            panic!("Not a file");
        }

        let source = File::open(path).unwrap();
        let entries = list_archive_files(&source)
            .unwrap()
            .iter()
            .map(PathBuf::from)
            .collect();

        Self {
            db: db.clone(),
            cfg: cfg.clone(),
            game: game.clone(),
            name: name.to_string(),
            archive_path: path.to_path_buf(),
            entries,
            root: None,
        }
    }

    pub async fn with_root(&mut self, path: &Path) -> &Self {
        if !self.entries.iter().any(|e| e.starts_with(path)) {
            panic!("Invalid root path");
        }

        self.root = Some(path.to_path_buf());

        self
    }

    // TODO: Wrap in transaction
    pub async fn import(&self) -> Mod {
        let mod_ = add_mod(&self.db, &self.cfg, &self.game, &self.name).await;

        let archive_file = File::open(&self.archive_path).unwrap();
        let dest = mod_.dir().await.unwrap();

        if let Some(root) = &self.root {
            let staging_dir = tempdir().unwrap();

            uncompress_archive(archive_file, staging_dir.path(), Ownership::Ignore).unwrap();

            let source_path = staging_dir.path().join(root);

            move_directory(source_path, dest, DirectoryMoveOptions::default()).unwrap();
        } else {
            uncompress_archive(archive_file, &dest, Ownership::Ignore).unwrap();
        }

        mod_
    }
}

/// Insert mod to database and create an empty directory in the library
async fn add_mod(db: &Db, cfg: &Cfg, game: &Game, name: &str) -> Mod {
    let model = ActiveModel {
        name: Set(name.to_string()),
        game_id: Set(game.id()),
        ..Default::default()
    };

    let id = Entity::insert(model)
        .exec(db.conn())
        .await
        .unwrap()
        .last_insert_id;

    let mod_ = Mod::from_id(id, db, cfg);

    let dir = mod_.dir().await.unwrap();
    create_dir_all(dir).unwrap();

    mod_
}

#[cfg(test)]
mod test {
    use std::{
        fs::{self, File},
        io::Write,
        path::Path,
    };

    use tempfile::tempdir;
    use zip::{ZipWriter, write::SimpleFileOptions};

    use crate::{Repository, repository::DeployKind};

    use super::*;

    #[tokio::test]
    async fn create_adds_empty_mod_directory() {
        let repo = Repository::in_memory().await;
        let game = repo
            .add_game("Morrowind", DeployKind::OpenMW)
            .await
            .unwrap();

        let builder = ModBuilder::new(&repo.db, &repo.cfg, &game, "Patch for Purists");
        let mod_ = builder.create().await;
        let dir = mod_.dir().await.unwrap();

        assert!(dir.exists());
        assert!(dir.is_dir());
        assert_eq!(game.mods().await.unwrap(), vec![mod_]);
    }

    #[tokio::test]
    async fn from_dir_copies_source_contents_into_mod_directory() {
        let repo = Repository::in_memory().await;
        let game = repo
            .add_game("Morrowind", DeployKind::OpenMW)
            .await
            .unwrap();
        let source = tempdir().unwrap();

        fs::create_dir_all(source.path().join("meshes")).unwrap();
        fs::write(source.path().join("meshes").join("marker.nif"), "mesh").unwrap();

        let builder = ModBuilder::new(&repo.db, &repo.cfg, &game, "Mesh Replacer");
        let mod_ = builder.from_dir(source.path()).await;
        let dir = mod_.dir().await.unwrap();

        assert!(dir.join("meshes").join("marker.nif").is_file());
        assert!(!dir.join(source.path().file_name().unwrap()).exists());
    }

    #[tokio::test]
    async fn archive_import_with_root_strips_selected_root_directory() {
        let repo = Repository::in_memory().await;
        let game = repo
            .add_game("Morrowind", DeployKind::OpenMW)
            .await
            .unwrap();
        let archive_dir = tempdir().unwrap();
        let archive_path = archive_dir.path().join("wrapped.zip");

        write_zip(
            &archive_path,
            &[
                ("FooMod/meshes/marker.nif", "mesh"),
                ("FooMod/textures/marker.dds", "texture"),
            ],
        );

        let builder = ModBuilder::new(&repo.db, &repo.cfg, &game, "Wrapped Mod");
        let mut import = builder.from_archive("Wrapped Mod", &archive_path).await;

        import.with_root(Path::new("FooMod")).await;

        let mod_ = import.import().await;
        let dir = mod_.dir().await.unwrap();

        assert!(dir.join("meshes").join("marker.nif").is_file());
        assert!(dir.join("textures").join("marker.dds").is_file());
        assert!(!dir.join("FooMod").exists());
    }

    fn write_zip(path: &Path, entries: &[(&str, &str)]) {
        let file = File::create(path).unwrap();
        let mut zip = ZipWriter::new(file);
        let options = SimpleFileOptions::default();

        for (path, contents) in entries {
            zip.start_file(path, options).unwrap();
            zip.write_all(contents.as_bytes()).unwrap();
        }

        zip.finish().unwrap();
    }
}
