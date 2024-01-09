use sparkle_convenience::{
    error::IntoError,
    interaction::{extract::InteractionExt, InteractionHandle},
};
use twilight_interactions::command::CreateCommand;
use twilight_model::{application::interaction::Interaction, id::Id};

use crate::{Context, Error};

mod adminbal;

#[derive(Debug)]
struct InteractionContext<'ctx> {
    ctx: &'ctx Context,
    handle: InteractionHandle<'ctx>,
    interaction: Interaction,
}

impl<'ctx> InteractionContext<'ctx> {
    async fn handle(self) -> Result<(), anyhow::Error> {
        match self.interaction.name().ok()? {
            adminbal::Command::NAME => self.handle_adminbal_command().await,
            _ => Err(Error::UnknownInteraction(self.interaction).into()),
        }
    }
}

impl Context {
    pub async fn create_commands(&self) -> Result<(), anyhow::Error> {
        let commands = [
            adminbal::Command::create_command().into(),
        ];

        self.bot
            .interaction_client()
            .set_guild_commands(
                Id::new(self.config.debug_scope),
                &commands,
            )
            .await?;
        tracing::info!("Created guild commands");
        Ok(())
    }

    pub async fn handle_interaction(&self, interaction: Interaction) {
        let handle = self.bot.interaction_handle(&interaction);
        let ctx = InteractionContext {
            ctx: self,
            handle: handle.clone(),
            interaction,
        };

        if let Err(err) = ctx.handle().await {
            tracing::error!("Error handling interaction:\n {}", err.backtrace());
        }
    }
}
