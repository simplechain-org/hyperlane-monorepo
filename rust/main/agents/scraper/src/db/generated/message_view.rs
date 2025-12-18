//! `SeaORM` Entity for message_view
//! 跨链消息视图，包含完整的跨链交易信息

use sea_orm::entity::prelude::*;

#[derive(Copy, Clone, Default, Debug, DeriveEntity)]
pub struct Entity;

impl EntityName for Entity {
    fn table_name(&self) -> &str {
        "message_view"
    }
}

/// message_view 视图模型
/// 包含跨链交易的完整信息，包括发送时间、投递时间、交易哈希等
#[derive(Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel)]
pub struct Model {
    /// 消息数据库 ID
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i64,
    /// 消息 ID（链上唯一标识）
    pub msg_id: Vec<u8>,
    /// 消息 nonce
    pub nonce: i32,
    /// 是否已投递
    pub is_delivered: bool,
    /// Gas 支付次数 (COUNT 返回 INT8)
    pub num_payments: i64,
    /// 总支付金额 (SUM 返回 NUMERIC/Decimal)
    pub total_payment: BigDecimal,
    /// 总 Gas 数量 (SUM 返回 NUMERIC/Decimal)
    pub total_gas_amount: BigDecimal,

    /// 源链 domain ID
    pub origin_domain_id: i32,
    /// 源链 chain ID
    pub origin_chain_id: Option<i64>,
    /// 源链名称
    pub origin_domain: Option<String>,

    /// 目标链 domain ID
    pub destination_domain_id: i32,
    /// 目标链 chain ID
    pub destination_chain_id: Option<i64>,
    /// 目标链名称
    pub destination_domain: Option<String>,

    /// 发送抓取时间
    pub send_scraped_at: TimeDateTime,
    /// 发送发生时间（区块时间）
    pub send_occurred_at: Option<TimeDateTime>,
    /// 投递抓取时间
    pub delivery_scraped_at: Option<TimeDateTime>,
    /// 投递发生时间（区块时间）
    pub delivery_occurred_at: Option<TimeDateTime>,

    /// 发送者地址（合约地址）
    pub sender: Vec<u8>,
    /// 接收者地址
    pub recipient: Vec<u8>,
    /// 源链 mailbox 地址
    pub origin_mailbox: Vec<u8>,
    /// 目标链 mailbox 地址
    pub destination_mailbox: Option<Vec<u8>>,

    /// 源链交易 ID
    pub origin_tx_id: i64,
    /// 源链交易哈希
    pub origin_tx_hash: Option<Vec<u8>>,
    /// 源链交易发送者（实际发起跨链交易的用户地址）
    pub origin_tx_sender: Option<Vec<u8>>,
    /// 源链区块高度
    pub origin_block_height: Option<i64>,
    /// 源链区块哈希
    pub origin_block_hash: Option<Vec<u8>>,

    /// 目标链交易 ID
    pub destination_tx_id: Option<i64>,
    /// 目标链交易哈希
    pub destination_tx_hash: Option<Vec<u8>>,
    /// 目标链区块高度
    pub destination_block_height: Option<i64>,
    /// 目标链区块哈希
    pub destination_block_hash: Option<Vec<u8>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
pub enum Column {
    Id,
    MsgId,
    Nonce,
    IsDelivered,
    NumPayments,
    TotalPayment,
    TotalGasAmount,
    OriginDomainId,
    OriginChainId,
    OriginDomain,
    DestinationDomainId,
    DestinationChainId,
    DestinationDomain,
    SendScrapedAt,
    SendOccurredAt,
    DeliveryScrapedAt,
    DeliveryOccurredAt,
    Sender,
    Recipient,
    OriginMailbox,
    DestinationMailbox,
    OriginTxId,
    OriginTxHash,
    OriginTxSender,
    OriginBlockHeight,
    OriginBlockHash,
    DestinationTxId,
    DestinationTxHash,
    DestinationBlockHeight,
    DestinationBlockHash,
}

#[derive(Copy, Clone, Debug, EnumIter, DerivePrimaryKey)]
pub enum PrimaryKey {
    Id,
}

impl PrimaryKeyTrait for PrimaryKey {
    type ValueType = i64;
    fn auto_increment() -> bool {
        false
    }
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {}

impl ColumnTrait for Column {
    type EntityName = Entity;
    fn def(&self) -> ColumnDef {
        match self {
            Self::Id => ColumnType::BigInteger.def(),
            Self::MsgId => ColumnType::VarBinary(StringLen::None).def(),
            Self::Nonce => ColumnType::Integer.def(),
            Self::IsDelivered => ColumnType::Boolean.def(),
            Self::NumPayments => ColumnType::BigInteger.def(),
            Self::TotalPayment => ColumnType::Decimal(Some((78u32, 0u32))).def(),
            Self::TotalGasAmount => ColumnType::Decimal(Some((78u32, 0u32))).def(),
            Self::OriginDomainId => ColumnType::Integer.def(),
            Self::OriginChainId => ColumnType::BigInteger.def().null(),
            Self::OriginDomain => ColumnType::String(StringLen::None).def().null(),
            Self::DestinationDomainId => ColumnType::Integer.def(),
            Self::DestinationChainId => ColumnType::BigInteger.def().null(),
            Self::DestinationDomain => ColumnType::String(StringLen::None).def().null(),
            Self::SendScrapedAt => ColumnType::DateTime.def(),
            Self::SendOccurredAt => ColumnType::DateTime.def().null(),
            Self::DeliveryScrapedAt => ColumnType::DateTime.def().null(),
            Self::DeliveryOccurredAt => ColumnType::DateTime.def().null(),
            Self::Sender => ColumnType::VarBinary(StringLen::None).def(),
            Self::Recipient => ColumnType::VarBinary(StringLen::None).def(),
            Self::OriginMailbox => ColumnType::VarBinary(StringLen::None).def(),
            Self::DestinationMailbox => ColumnType::VarBinary(StringLen::None).def().null(),
            Self::OriginTxId => ColumnType::BigInteger.def(),
            Self::OriginTxHash => ColumnType::VarBinary(StringLen::None).def().null(),
            Self::OriginTxSender => ColumnType::VarBinary(StringLen::None).def().null(),
            Self::OriginBlockHeight => ColumnType::BigInteger.def().null(),
            Self::OriginBlockHash => ColumnType::VarBinary(StringLen::None).def().null(),
            Self::DestinationTxId => ColumnType::BigInteger.def().null(),
            Self::DestinationTxHash => ColumnType::VarBinary(StringLen::None).def().null(),
            Self::DestinationBlockHeight => ColumnType::BigInteger.def().null(),
            Self::DestinationBlockHash => ColumnType::VarBinary(StringLen::None).def().null(),
        }
    }
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        panic!("No relations defined for message_view")
    }
}

impl ActiveModelBehavior for ActiveModel {}
