use std::{fs::create_dir_all, iter::once, path::Path};

use damascus::{Filesystem, FuseOverlayFs};

use crate::{
    data::{games::Game, profiles::Profile},
    deployers::Deploy,
};

#[derive(Debug)]
pub struct OverlayDeployer {
    overlay: FuseOverlayFs,
}

impl Deploy for OverlayDeployer {
    fn setup(game: &Game, profile: &Profile) -> Self {
        let profile_dir = game.dir().join(profile.name());

        // Initialize overlay directories if missing
        let overlay_dir = profile_dir.join("overlay");
        create_dir_all(overlay_dir.join("work")).unwrap();
        create_dir_all(overlay_dir.join("upper")).unwrap();

        let work_dir = overlay_dir.join("work");
        let upper_dir = overlay_dir.join("upper");

        let resolved_mod_entries = profile.resolve_mod_entries(game);
        let lower_dirs: Vec<&Path> = once(game.game_dir())
            .chain(resolved_mod_entries.iter().map(|m| m.mod_ref().path()))
            .collect();

        let overlay = FuseOverlayFs::new(
            lower_dirs.into_iter(),
            Some(upper_dir),
            Some(work_dir),
            game.game_dir(),
            true,
        )
        .unwrap();

        Self { overlay }
    }

    fn deploy(&mut self) {
        self.overlay.mount().unwrap();
    }

    fn undeploy(&mut self) {
        self.overlay.unmount().unwrap();
    }
}
