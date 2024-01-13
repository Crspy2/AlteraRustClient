use sparkle_convenience::{
    error::IntoError,
    interaction::{extract::InteractionExt, InteractionHandle},
};

use twilight_interactions::command::CreateCommand;
use twilight_model::{application::interaction::Interaction, id::Id};

use crate::{Context, Error};

mod adminbal;
mod balance;
mod search;
mod userdata;

#[derive(Debug)]
struct InteractionContext<'ctx> {
    ctx: &'ctx Context,
    handle: InteractionHandle<'ctx>,
    interaction: Interaction,
}

impl<'ctx> InteractionContext<'ctx> {
    async fn handle(self) -> Result<(), anyhow::Error> {
        tracing::info!("Processing command {}", &self.interaction.name().ok()?);

        match self.interaction.name().ok()? {
            adminbal::AdminBalCommand::NAME => self.handle_adminbal_command().await,
            balance::BalanceCommand::NAME => self.handle_balance_command().await,
            search::SearchCommand::NAME => self.handle_search_command().await,
            userdata::UserDataCommand::NAME => self.handle_user_data_command().await,
            _ => Err(Error::UnknownInteraction(self.interaction).into()),
        }
    }
}

impl Context {
    pub async fn create_commands(&self) -> Result<(), anyhow::Error> {
        let commands = [
            adminbal::AdminBalCommand::create_command().into(),
            balance::BalanceCommand::create_command().into(),
            search::SearchCommand::create_command().into(),
            userdata::UserDataCommand::create_command().into(),
        ];

        self.bot
            .interaction_client()
            .set_guild_commands(Id::new(self.config.debug_scope), &commands)
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
