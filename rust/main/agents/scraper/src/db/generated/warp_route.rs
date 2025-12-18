//! `SeaORM` Entity for warp_route table
//! 资产跨链桥路由表

use sea_orm::entity::prelude::*;

#[derive(Copy, Clone, Default, Debug, DeriveEntity)]
pub struct Entity;

impl EntityName for Entity {
    fn table_name(&self) -> &str {
        "warp_route"
    }
}

#[derive(Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel, Eq)]
pub struct Model {
    /// 自增主键
    pub id: i64,
    /// 创建时间
    pub time_created: TimeDateTime,
    /// 更新时间
    pub time_updated: TimeDateTime,
    /// 源链 ID
    pub src_chain_id: i64,
    /// 源链合约地址（20 字节 EVM 地址）
    pub src_chain_address: Vec<u8>,
    /// 目标链 ID
    pub dst_chain_id: i64,
    /// 目标链合约地址（20 字节 EVM 地址）
    pub dst_chain_address: Vec<u8>,
    /// 代币名称
    pub token_name: String,
    /// 代币符号
    pub token_symbol: String,
    /// 代币精度
    pub token_decimals: i16,
    /// 是否启用
    pub is_enabled: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
pub enum Column {
    Id,
    TimeCreated,
    TimeUpdated,
    SrcChainId,
    SrcChainAddress,
    DstChainId,
    DstChainAddress,
    TokenName,
    TokenSymbol,
    TokenDecimals,
    IsEnabled,
}

#[derive(Copy, Clone, Debug, EnumIter, DerivePrimaryKey)]
pub enum PrimaryKey {
    Id,
}

impl PrimaryKeyTrait for PrimaryKey {
    type ValueType = i64;
    fn auto_increment() -> bool {
        true
    }
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {}

impl ColumnTrait for Column {
    type EntityName = Entity;
    fn def(&self) -> ColumnDef {
        match self {
            Self::Id => ColumnType::BigInteger.def(),
            Self::TimeCreated => ColumnType::DateTime.def(),
            Self::TimeUpdated => ColumnType::DateTime.def(),
            Self::SrcChainId => ColumnType::BigInteger.def(),
            Self::SrcChainAddress => ColumnType::VarBinary(StringLen::None).def(),
            Self::DstChainId => ColumnType::BigInteger.def(),
            Self::DstChainAddress => ColumnType::VarBinary(StringLen::None).def(),
            Self::TokenName => ColumnType::String(StringLen::None).def(),
            Self::TokenSymbol => ColumnType::String(StringLen::None).def(),
            Self::TokenDecimals => ColumnType::SmallInteger.def(),
            Self::IsEnabled => ColumnType::Boolean.def(),
        }
    }
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        panic!("No relations defined for WarpRoute")
    }
}

impl ActiveModelBehavior for ActiveModel {}
