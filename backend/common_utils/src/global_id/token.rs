use crate::global_id::CellId;

crate::global_id_type!(
    GlobalTokenId,
    "A global id that can be used to identify a token.

The format will be `<cell_id>_<entity_prefix>_<time_ordered_id>`.

Example: `cell1_tok_uu1a2b3c4d5e6f7g8h9i0j1k2l3m4n5o6p`"
);

impl GlobalTokenId {
    /// Get string representation of the id
    pub fn get_string_repr(&self) -> &str {
        self.0.get_string_repr()
    }

    /// Generate a new GlobalTokenId from a cell id
    pub fn generate(cell_id: &CellId) -> Self {
        let global_id = super::GlobalId::generate(cell_id, super::GlobalEntity::Token);
        Self(global_id)
    }
}

impl crate::events::ApiEventMetric for GlobalTokenId {
    fn get_api_event_type(&self) -> Option<crate::events::ApiEventsType> {
        Some(crate::events::ApiEventsType::Token {
            token_id: Some(self.clone()),
        })
    }
}
