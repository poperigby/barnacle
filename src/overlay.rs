use damascus::{Filesystem, FuseOverlayFs};

use crate::{games::Game, profiles::Profile};

pub struct Overlay {
    handle: FuseOverlayFs,
}

impl Overlay {
    pub fn new(game: &Game, profile: &Profile) -> Self {
        let overlay_dir = game.dir().join(profile.name()).join("overlay");

        let work_dir = overlay_dir.join("work");
        let upper_dir = overlay_dir.join("upper");

        let enabled_mods: Vec<_> = profile
            .mod_ids()
            .iter()
            .filter_map(|id| game.mods().get(id))
            .collect();

        let mut lower_dirs = vec![game.game_dir().to_path_buf()];
        lower_dirs.extend(enabled_mods.iter().map(|m| m.dir()));

        let handle = FuseOverlayFs::new(
            lower_dirs.iter().map(|p| p.as_path()),
            Some(upper_dir),
            Some(work_dir),
            game.game_dir(),
            false,
        )
        .unwrap();

        Self { handle }
    }

    pub fn mount(&mut self) {
        self.handle.mount().unwrap();
    }

    pub fn unmount(&mut self) {
        self.handle.unmount().unwrap();
    }
}
