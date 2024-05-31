use std::borrow::Cow;

use getset::CopyGetters;

use self::daddy::Daddy;
use crate::card::{Action, Card, CardDefinition, Deck, Nickname};
use crate::op::{Op, OpError, OpErrorUtils, OpImplResult, OpPlugin, OpRegistrar};
use crate::prelude::*;

pub const MAX_MON: u32 = 100_000_000;

pub mod key {
    use bevy::ecs::entity::Entity;
    use typed_key::{typed_key, Key};

    pub const CARD_ID: Key<Entity> = typed_key!("card_id");
    pub const NEW_CARD: Key<bool> = typed_key!("new_card");
}

#[derive(Debug, Default)]
pub struct ItemPlugin;

impl Plugin for ItemPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Daddy<Card>>()
            .register_type::<Item>()
            .add_plugins(OpPlugin::<ItemOp>::default());
    }
}

#[derive(Component, CopyGetters, Debug, Default, Reflect)]
#[get_copy = "pub"]
pub struct Wallet {
    mon: u32,
}

impl Wallet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_mon(mut self, mon: u32) -> Self {
        self.mon = mon;
        self
    }

    pub fn increase_mon(&mut self, mon: u32) {
        self.mon = self.mon.saturating_add(mon).min(MAX_MON);
    }

    pub fn decrease_mon(&mut self, mon: u32) {
        self.mon = self.mon.saturating_sub(mon);
    }

    /// Decrease mon only if we have sufficient amount
    pub fn try_spend(&mut self, mon: u32) -> bool {
        if self.mon >= mon {
            self.mon -= mon;
            true
        } else {
            false
        }
    }
}

#[derive(Clone, Debug, Reflect, PartialEq)]
pub enum Item {
    Card(Handle<CardDefinition>),
    Mon(u32), // Others?
}

impl Item {
    pub fn name(&self, cards: &Assets<CardDefinition>) -> Cow<str> {
        match self {
            Self::Mon(_) => Cow::from("Mon"),
            Self::Card(handle) => cards
                .get(handle)
                .map(|card_def| Cow::Owned(card_def.id().to_owned()))
                .unwrap_or_else(|| {
                    log::error!("Unable to retreive name for card {handle:?}");
                    Cow::from("???")
                }),
        }
    }

    pub fn description(&self, cards: &Assets<CardDefinition>) -> Cow<str> {
        match self {
            Self::Mon(_) => Cow::from("Makes the world go round"), // TODO Better money description
            Self::Card(handle) => cards
                .get(handle)
                .map(|card_def| Cow::Owned(card_def.description().to_owned()))
                .unwrap_or_else(|| {
                    log::error!("Unable to retreive description for card {handle:?}");
                    Cow::from("???")
                }),
        }
    }

    pub fn actions(&self, cards: &Assets<CardDefinition>) -> Vec<Handle<Action>> {
        match self {
            Self::Mon(_) => Vec::default(), // TODO Better money description
            Self::Card(handle) => cards
                .get(handle)
                .map(|card_def| card_def.actions().clone())
                .unwrap_or_else(|| {
                    log::error!("Unable to retreive description for card {handle:?}");
                    Vec::default()
                }),
        }
    }

    pub fn action_names(
        &self,
        cards: &Assets<CardDefinition>,
        actions: &Assets<Action>,
    ) -> Vec<Cow<str>> {
        match self {
            Self::Mon(_) => Vec::default(), // TODO Better money description
            Self::Card(handle) => cards
                .get(handle)
                .map(|card_def| {
                    let card_actions = card_def.actions();

                    card_actions
                        .iter()
                        .map(|action_handle| {
                            actions
                                .get(action_handle)
                                .map(|a| a.id_cow())
                                .unwrap_or(Cow::Borrowed("???"))
                        })
                        .collect()
                })
                .unwrap_or_else(|| {
                    log::error!("Unable to retreive description for card {handle:?}");
                    Vec::default()
                }),
        }
    }
}

#[derive(Debug, Reflect)]
pub enum ItemOp {
    AddItem { item: Item, refund: u32 },
    GiveItem { item: Item, target: Entity }, // Remove #[non_exhaustive] when we add another enum
                                             // Give, Drop, Trash?
}

impl Op for ItemOp {
    fn register_systems(mut registrar: OpRegistrar<Self>) {
        registrar.register_op(opsys_add_item);
    }

    fn system_index(&self) -> usize {
        0
    }
}

pub fn opsys_add_item(
    In((source_id, op)): In<(Entity, ItemOp)>,
    mut commands: Commands,
    res_daddy_card: Res<Daddy<Card>>,
    mut q_deck: Query<&mut Deck>,
    mut q_wallet: Query<&mut Wallet>,
    q_card: Query<&Handle<CardDefinition>, Without<Nickname>>,
) -> OpImplResult {
    if let ItemOp::AddItem { item, refund } = op {
        match item {
            Item::Card(card_handle) => {
                let interim_result = (|| {
                    let mut deck = q_deck.get_mut(source_id).invalid()?;

                    let existing_card = deck
                        .cards_iter()
                        .filter_map(|card_id| {
                            let card = q_card.get(card_id).ok()?;
                            if card == &card_handle {
                                Some(card_id)
                            } else {
                                None
                            }
                        })
                        .next();
                    let mut metadata = Metadata::default();
                    let card_id = if let Some(existing_card_id) = existing_card {
                        metadata.put(key::NEW_CARD, false).invalid()?;
                        existing_card_id
                    } else {
                        metadata.put(key::NEW_CARD, true).invalid()?;
                        // TODO source individual parent or component
                        commands
                            .spawn((card_handle,))
                            .set_parent(**res_daddy_card)
                            .id()
                    };
                    metadata.put(key::CARD_ID, card_id).invalid()?;
                    deck.add_card(card_id);
                    Ok(default())
                })();
                // Refund them if problems occur
                if interim_result.is_err() {
                    let mut wallet = q_wallet.get_mut(source_id).critical()?;
                    wallet.increase_mon(refund);
                }
                interim_result
            },
            Item::Mon(mon) => {
                let mut wallet = q_wallet.get_mut(source_id).invalid()?;
                wallet.increase_mon(mon);
                Ok(default())
            },
        }
    } else {
        Err(OpError::MismatchedOpSystem)
    }
}
