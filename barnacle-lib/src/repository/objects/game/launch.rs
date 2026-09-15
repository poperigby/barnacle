use super::*;

impl Game {
    pub async fn launch(&self) {
        match self.deploy_kind().await.unwrap() {
            DeployKind::OpenMW => {
                let mut command = deployers::openmw::OpenMw::prepare(self).await;

                command.spawn().unwrap();
            }
            _ => println!("I DON'T UNDERSTAND THIS DEPLOY TYPE :("),
        };
    }

    pub(crate) async fn launch_command(&self) -> Command {
        let model = self.model(self.db.conn()).await.unwrap();

        let program = model.launch_program;
        let args = model.launch_args.split(" ");

        let mut command = Command::new(program);

        command.args(args);

        command
    }
}
