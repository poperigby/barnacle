use damascus::{Filesystem, FuseOverlayFs};

use crate::{deployers::Deploy, games::Game, profiles::Profile};

#[derive(Debug)]
pub struct GenericDeployer {
    overlay: FuseOverlayFs,
}

impl Deploy for GenericDeployer {
    type T = Self;

    fn init(game: &Game, profile: &Profile) -> Self {
        let overlay_dir = game.dir().join(profile.name()).join("overlay");

        let work_dir = overlay_dir.join("work");
        let upper_dir = overlay_dir.join("upper");

        let enabled_mods = profile.resolve_mods(game);

        let mut lower_dirs = vec![game.game_dir().to_path_buf()];
        lower_dirs.extend(enabled_mods.iter().map(|m| m.mod_ref().dir()));

        let overlay = FuseOverlayFs::new(
            lower_dirs.iter().map(|p| p.as_path()),
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
