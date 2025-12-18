//! Migration file to create the warp_route table
//! Used to store asset cross-chain bridge route information

use sea_orm::prelude::TimeDateTime;
use time::OffsetDateTime;

use sea_orm_migration::prelude::*;

/// Mock warp routes data, used to initialize the database
/// Administrators can subsequently modify to correct information
/// Addresses use 20-byte EVM compatible format
const WARP_ROUTES: &[RawWarpRoute] = &[
    // ---------- Begin: Mock Warp Routes (example data) -------------
    // Ethereum <-> Arbitrum USDC
    RawWarpRoute {
        src_chain_id: 11155111,
        src_chain_address: "0x7ffF9d791D412d3c05ac44F4266146d5B68280d5",
        dst_chain_id: 1914,
        dst_chain_address: "0x449BD1a4c996F7D112fDB49F5a9A13f56faA2984",
        token_name: "Tether USD",
        token_symbol: "USDT",
        token_decimals: 6,
        is_enabled: true,
    },
    // Ethereum <-> Arbitrum USDT
    RawWarpRoute {
        src_chain_id: 1914,
        src_chain_address: "0x0653125658E9bfB8CCadAf410B3cb6C058117650",
        dst_chain_id: 71,
        dst_chain_address: "0x894d6add814b0821f1010241144b2c47b9e2d409",
        token_name: "Tether USD",
        token_symbol: "USDT",
        token_decimals: 6,
        is_enabled: true,
    },
];

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create warp_route table
        manager
            .create_table(
                Table::create()
                    .table(WarpRoute::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(WarpRoute::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(WarpRoute::TimeCreated)
                            .timestamp()
                            .not_null()
                            .default("NOW()"),
                    )
                    .col(
                        ColumnDef::new(WarpRoute::TimeUpdated)
                            .timestamp()
                            .not_null()
                            .default("NOW()"),
                    )
                    .col(
                        ColumnDef::new(WarpRoute::SrcChainId)
                            .big_unsigned()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(WarpRoute::SrcChainAddress)
                            .binary()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(WarpRoute::DstChainId)
                            .big_unsigned()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(WarpRoute::DstChainAddress)
                            .binary()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(WarpRoute::TokenName)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(WarpRoute::TokenSymbol)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(WarpRoute::TokenDecimals)
                            .small_unsigned()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(WarpRoute::IsEnabled)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .to_owned(),
            )
            .await?;

        // Create index: source chain address index
        manager
            .create_index(
                Index::create()
                    .name("idx_warp_route_src_chain_address")
                    .table(WarpRoute::Table)
                    .col(WarpRoute::SrcChainId)
                    .col(WarpRoute::SrcChainAddress)
                    .to_owned(),
            )
            .await?;

        // Create index: destination chain address index
        manager
            .create_index(
                Index::create()
                    .name("idx_warp_route_dst_chain_address")
                    .table(WarpRoute::Table)
                    .col(WarpRoute::DstChainId)
                    .col(WarpRoute::DstChainAddress)
                    .to_owned(),
            )
            .await?;

        // Insert initial data
        use sea_orm_migration::sea_orm::ActiveValue::Set;
        use sea_orm_migration::sea_orm::EntityTrait;

        let db = manager.get_connection();
        for route in WARP_ROUTES {
            let now = {
                let offset = OffsetDateTime::now_utc();
                TimeDateTime::new(offset.date(), offset.time())
            };

            // Parse address string to byte array
            let src_address = parse_hex_address(route.src_chain_address);
            let dst_address = parse_hex_address(route.dst_chain_address);

            EntityTrait::insert(warp_route::ActiveModel {
                id: Default::default(),
                time_created: Set(now),
                time_updated: Set(now),
                src_chain_id: Set(route.src_chain_id),
                src_chain_address: Set(src_address),
                dst_chain_id: Set(route.dst_chain_id),
                dst_chain_address: Set(dst_address),
                token_name: Set(route.token_name.to_owned()),
                token_symbol: Set(route.token_symbol.to_owned()),
                token_decimals: Set(route.token_decimals),
                is_enabled: Set(route.is_enabled),
            })
            .exec(db)
            .await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(WarpRoute::Table).to_owned())
            .await
    }
}

/// Parse hexadecimal address string to byte array (20-byte EVM address)
fn parse_hex_address(address: &str) -> Vec<u8> {
    let hex_str = address.strip_prefix("0x").unwrap_or(address);
    hex::decode(hex_str).unwrap_or_else(|_| vec![0u8; 20])
}

/// WarpRoute table definition
#[derive(Iden)]
pub enum WarpRoute {
    Table,
    /// Auto-increment primary key
    Id,
    /// Creation time
    TimeCreated,
    /// Update time
    TimeUpdated,
    /// Source chain ID
    SrcChainId,
    /// Source chain contract address
    SrcChainAddress,
    /// Destination chain ID
    DstChainId,
    /// Destination chain contract address
    DstChainAddress,
    /// Token name
    TokenName,
    /// Token symbol
    TokenSymbol,
    /// Token decimals
    TokenDecimals,
    /// Whether enabled
    IsEnabled,
}

/// SeaORM Entity definition, used for inserting data
mod warp_route {
    use sea_orm_migration::sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "warp_route")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub time_created: TimeDateTime,
        pub time_updated: TimeDateTime,
        pub src_chain_id: u64,
        pub src_chain_address: Vec<u8>,
        pub dst_chain_id: u64,
        pub dst_chain_address: Vec<u8>,
        pub token_name: String,
        pub token_symbol: String,
        pub token_decimals: u16,
        pub is_enabled: bool,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

/// Raw WarpRoute data structure
struct RawWarpRoute {
    src_chain_id: u64,
    src_chain_address: &'static str,
    dst_chain_id: u64,
    dst_chain_address: &'static str,
    token_name: &'static str,
    token_symbol: &'static str,
    token_decimals: u16,
    is_enabled: bool,
}