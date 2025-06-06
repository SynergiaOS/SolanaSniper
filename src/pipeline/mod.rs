/*!
🧠 Pipeline Module - Orchestrates the complete trading intelligence flow

This module contains the core pipeline controller that orchestrates:
- Soul Meteor Scanner (quantitative opportunity detection)
- Crawl4AI Validator (qualitative sentiment validation)  
- Trading Decision Engine (strategy selection and execution)
*/

pub mod controller;
pub mod opportunity;
pub mod decision_engine;

pub use controller::PipelineController;
pub use opportunity::{ValidatedOpportunity, OpportunityStatus};
pub use decision_engine::{TradingDecision, DecisionEngine};
