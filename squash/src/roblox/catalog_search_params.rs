use super::prelude::*;

#[cfg_attr(feature = "serde", derive(Serialize, ReverseDeserialize))]
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct CatalogSearchParams {
    pub include_off_sale: bool,
    pub limit: u8,
    pub min_price: u32,
    pub max_price: u32,
    pub creator_name: String,
    pub search_keyworld: String,
    pub sort_type: EnumItem,
    pub sort_aggregration: EnumItem,
    pub category_filter: EnumItem,
    pub sales_type_filter: EnumItem,
    pub asset_types: Vec<EnumItem>,
}
impl_squash_object_a!(CatalogSearchParams, include_off_sale, limit, min_price, max_price, creator_name, search_keyworld, sort_type, sort_aggregration, category_filter, sales_type_filter, asset_types;asset_types, sales_type_filter, category_filter, sort_aggregration, sort_type, search_keyworld, creator_name, max_price, min_price, limit, include_off_sale);
