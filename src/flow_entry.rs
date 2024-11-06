use std::{collections::HashMap, fmt::Display};

use serde::{Deserialize, Serialize};

// Struct for the "match" object
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Match {
    pub in_port: String,
}
impl Display for Match {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("{}", self.in_port).as_str())
    }
}

// Struct for the "instruction_apply_actions" object within "instructions"
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InstructionApplyActions {
    pub actions: String,
}
impl Display for InstructionApplyActions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("{}", self.actions).as_str())
    }
}

// Struct for the "instructions" object
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Instructions {
    pub instruction_apply_actions: InstructionApplyActions,
}
impl Display for Instructions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("{}", self.instruction_apply_actions).as_str())
    }
}

// Struct for each flow-mod entry, which includes version, command, and other fields
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FlowMod {
    pub version: String,
    pub command: String,
    pub cookie: String,
    pub priority: String,
    pub idleTimeoutSec: String,
    pub hardTimeoutSec: String,
    pub outPort: String,
    pub flags: String,
    pub cookieMask: String,
    pub outGroup: String,
    pub tableId: String,
    #[serde(rename = "match")]
    pub match_field: Match,
    pub instructions: Instructions,
}

impl Display for FlowMod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("{}, {}", self.match_field, self.instructions).as_str())
    }
}

// Define a HashMap for the JSON structure, where each entry has a unique name
pub type FlowModEntries = HashMap<String, FlowMod>;

// Define the main struct to hold the switch data, where the switch ID maps to an array of flow entries
pub type SwitchData = HashMap<String, Vec<FlowModEntries>>;

#[derive(Debug, Serialize, Deserialize)]
pub struct FlowRule {
    name: String,
    switch: String,
    priority: String,
    in_port: Option<String>, // Match field
    actions: String,         // Actions to apply (e.g., "output=2")
}

pub fn convert_to_flow_rules(switch_data: &SwitchData) -> Vec<FlowRule> {
    let mut flow_rules = Vec::new();

    // Iterate over each switch and its associated flow entries
    for (switch_dpid, flow_entries) in switch_data {
        for flow_entry in flow_entries {
            for (flow_name, flow_mod) in flow_entry {
                // Create a FlowRule from the current FlowMod entry
                let flow_rule = FlowRule {
                    name: flow_name.clone(),
                    switch: switch_dpid.clone(),
                    priority: flow_mod.priority.clone(),
                    in_port: Some(flow_mod.match_field.in_port.clone()),
                    actions: flow_mod
                        .instructions
                        .instruction_apply_actions
                        .actions
                        .clone(),
                };
                flow_rules.push(flow_rule);
            }
        }
    }

    flow_rules
}
