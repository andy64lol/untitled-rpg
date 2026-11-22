use std::collections::HashSet;

use crate::assets::Assets;
use crate::LOG;

#[derive(Clone, Debug)]
enum Node {
    Command { name: String, args: Vec<String> },
    If {
        condition: Vec<String>,
        then_body: Vec<Node>,
        else_body: Vec<Node>,
    },
}

#[derive(Clone, Debug)]
pub struct ScriptContext {
    pub health: i32,
    pub max_health: i32,
    pub dead: bool,
    pub moving: bool,
    pub facing: String,
    pub gold: i32,
    pub map: String,
    pub flags: HashSet<String>,
    pub items: std::collections::HashMap<String, i32>,
}

#[derive(Clone, Debug)]
pub enum ScriptAction {
    None,
    Dialogue(String),
    Give { id: String, count: i32 },
    Take { id: String, count: i32 },
    Gold(i32),
    SetGold(i32),
    Heal(i32),
    Damage(i32),
    SetHealth(i32),
    Spawn { id: String, col: Option<i32>, row: Option<i32> },
    Flag(String),
    Unflag(String),
    Run(String),
    End,
}

pub struct ScriptRunner {
    frames: Vec<(Vec<Node>, usize)>,
    nested: Vec<ScriptRunner>,
    wait: f32,
    waiting_dialogue: bool,
}

impl ScriptRunner {
    pub fn load(assets: &Assets, name: &str) -> Option<Self> {
        let requested = std::path::Path::new(name);
        let path = if requested.is_absolute() || requested.exists() {
            requested.to_path_buf()
        } else {
            assets.src().join("scripts").join(format!("{name}.urpgs"))
        };
        let text = std::fs::read_to_string(path).ok()?;
        let tokens = tokenize(&text);
        let mut parser = Parser { tokens, index: 0 };
        let nodes = parser.parse_block();
        Some(Self {
            frames: vec![(nodes, 0)],
            nested: Vec::new(),
            wait: 0.0,
            waiting_dialogue: false,
        })
    }

    pub fn update(
        &mut self,
        dt: f32,
        ctx: &ScriptContext,
        dialogue_open: bool,
    ) -> ScriptAction {
        if let Some(nested) = self.nested.last_mut() {
            let action = nested.update(dt, ctx, dialogue_open);
            if matches!(action, ScriptAction::None | ScriptAction::End) {
                if matches!(action, ScriptAction::End) || !nested.is_running() {
                    self.nested.pop();
                }
            }
            return action;
        }

        if self.wait > 0.0 {
            self.wait = (self.wait - dt).max(0.0);
            return ScriptAction::None;
        }
        if self.waiting_dialogue {
            if dialogue_open {
                return ScriptAction::None;
            }
            self.waiting_dialogue = false;
        }

        while let Some((nodes, index)) = self.frames.last_mut() {
            if *index >= nodes.len() {
                self.frames.pop();
                continue;
            }
            let node = nodes[*index].clone();
            *index += 1;
            match node {
                Node::If { condition, then_body, else_body } => {
                    let branch = if eval_condition(&condition, ctx) {
                        then_body
                    } else {
                        else_body
                    };
                    if !branch.is_empty() {
                        self.frames.push((branch, 0));
                    }
                }
                Node::Command { name, args } => {
                    match name.as_str() {
                        "wait" => {
                            self.wait = args.first()
                                .and_then(|v| v.parse().ok())
                                .unwrap_or(0.0f32)
                                .max(0.0f32);
                            return ScriptAction::None;
                        }
                        "dialogue" | "say" => {
                            self.waiting_dialogue = true;
                            return ScriptAction::Dialogue(args.join(" "));
                        }
                        "run" => {
                            return args.first().map(|s| ScriptAction::Run(s.clone()))
                                .unwrap_or(ScriptAction::None);
                        }
                        "end" => {
                            self.frames.clear();
                            return ScriptAction::End;
                        }
                        "give" => {
                            return ScriptAction::Give {
                                id: arg(&args, 0),
                                count: arg_i(&args, 1, 1),
                            };
                        }
                        "take" => {
                            return ScriptAction::Take {
                                id: arg(&args, 0),
                                count: arg_i(&args, 1, 1),
                            };
                        }
                        "gold" => return ScriptAction::Gold(arg_i(&args, 0, 0)),
                        "setgold" => return ScriptAction::SetGold(arg_i(&args, 0, 0)),
                        "heal" => return ScriptAction::Heal(arg_i(&args, 0, 0)),
                        "damage" => return ScriptAction::Damage(arg_i(&args, 0, 0)),
                        "sethealth" => return ScriptAction::SetHealth(arg_i(&args, 0, 0)),
                        "spawn" => {
                            return ScriptAction::Spawn {
                                id: arg(&args, 0),
                                col: args.get(1).and_then(|v| v.parse().ok()),
                                row: args.get(2).and_then(|v| v.parse().ok()),
                            };
                        }
                        "flag" => return ScriptAction::Flag(arg(&args, 0)),
                        "unflag" => return ScriptAction::Unflag(arg(&args, 0)),
                        _ => {
                            unsafe {
                                if let Some(log) = LOG.as_mut() {
                                    log.log(&format!("[script] unknown command: {name}"));
                                }
                            }
                        }
                    }
                }
            }
        }
        ScriptAction::End
    }

    fn is_running(&self) -> bool {
        !self.frames.is_empty()
    }
}

fn arg(args: &[String], index: usize) -> String {
    args.get(index).cloned().unwrap_or_default()
}

fn arg_i(args: &[String], index: usize, default: i32) -> i32 {
    args.get(index).and_then(|v| v.parse().ok()).unwrap_or(default)
}

fn tokenize(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    let specials = "{}();<>!,";
    let operators = [">=", "<=", "==", "!=", ">", "<"];
    while i < chars.len() {
        let c = chars[i];
        if c == '\n' {
            tokens.push("NL".into());
            i += 1;
        } else if c.is_whitespace() || c == ',' {
            i += 1;
        } else if c == '#' || (c == '/' && i + 1 < chars.len() && chars[i + 1] == '/') {
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
        } else if c == '"' || c == '\'' {
            let quote = c;
            i += 1;
            let mut value = String::new();
            while i < chars.len() && chars[i] != quote {
                if chars[i] == '\\' && i + 1 < chars.len() {
                    value.push(chars[i + 1]);
                    i += 2;
                } else {
                    value.push(chars[i]);
                    i += 1;
                }
            }
            if i < chars.len() {
                i += 1;
            }
            tokens.push(value);
        } else if specials.contains(c) {
            let pair: String = chars[i..(i + 2).min(chars.len())].iter().collect();
            if operators.contains(&pair.as_str()) {
                tokens.push(pair);
                i += 2;
            } else {
                tokens.push(c.to_string());
                i += 1;
            }
        } else {
            let mut value = String::new();
            while i < chars.len()
                && !chars[i].is_whitespace()
                && !specials.contains(chars[i])
                && chars[i] != '#'
            {
                value.push(chars[i]);
                i += 1;
            }
            tokens.push(value);
        }
    }
    tokens
}

struct Parser {
    tokens: Vec<String>,
    index: usize,
}

impl Parser {
    fn peek(&self) -> Option<&str> {
        self.tokens.get(self.index).map(|s| s.as_str())
    }

    fn take(&mut self) -> String {
        let token = self.tokens[self.index].clone();
        self.index += 1;
        token
    }

    fn skip(&mut self) {
        while matches!(self.peek(), Some("NL") | Some(";")) {
            self.take();
        }
    }

    fn parse_block(&mut self) -> Vec<Node> {
        let mut nodes = Vec::new();
        self.skip();
        while let Some(token) = self.peek() {
            if token == "}" {
                break;
            }
            let first = self.take();
            if first.eq_ignore_ascii_case("if") {
                if self.peek() == Some("(") {
                    self.take();
                }
                let mut condition = Vec::new();
                while let Some(token) = self.peek() {
                    if token == ")" {
                        self.take();
                        break;
                    }
                    condition.push(self.take());
                }
                if self.peek() == Some("{") {
                    self.take();
                }
                let then_body = self.parse_block();
                if self.peek() == Some("}") {
                    self.take();
                }
                let mut else_body = Vec::new();
                if self.peek().map(|s| s.eq_ignore_ascii_case("else")).unwrap_or(false) {
                    self.take();
                    if self.peek().map(|s| s.eq_ignore_ascii_case("if")).unwrap_or(false) {
                        else_body = self.parse_block_after_else_if();
                    } else {
                        if self.peek() == Some("{") {
                            self.take();
                        }
                        else_body = self.parse_block();
                        if self.peek() == Some("}") {
                            self.take();
                        }
                    }
                }
                nodes.push(Node::If { condition, then_body, else_body });
            } else {
                let name = first.to_lowercase();
                let mut args = Vec::new();
                if self.peek() == Some("(") {
                    self.take();
                    while let Some(token) = self.peek() {
                        if token == ")" {
                            self.take();
                            break;
                        }
                        args.push(self.take());
                    }
                }
                while let Some(token) = self.peek() {
                    if matches!(token, "NL" | ";" | "{" | "}") {
                        break;
                    }
                    args.push(self.take());
                }
                self.skip();
                nodes.push(Node::Command { name, args });
            }
            self.skip();
        }
        nodes
    }

    fn parse_block_after_else_if(&mut self) -> Vec<Node> {
        self.take(); // "if"
        if self.peek() == Some("(") {
            self.take();
        }
        let mut condition = Vec::new();
        while let Some(token) = self.peek() {
            if token == ")" {
                self.take();
                break;
            }
            condition.push(self.take());
        }
        if self.peek() == Some("{") {
            self.take();
        }
        let then_body = self.parse_block();
        if self.peek() == Some("}") {
            self.take();
        }
        let mut else_body = Vec::new();
        if self.peek().map(|s| s.eq_ignore_ascii_case("else")).unwrap_or(false) {
            self.take();
            if self.peek() == Some("{") {
                self.take();
            }
            else_body = self.parse_block();
            if self.peek() == Some("}") {
                self.take();
            }
        }
        vec![Node::If { condition, then_body, else_body }]
    }
}

fn eval_condition(args: &[String], ctx: &ScriptContext) -> bool {
    if args.is_empty() {
        return false;
    }
    let kind = args[0].to_lowercase();
    let (value, offset): (ConditionValue, usize) = if kind.starts_with("player.") {
        match kind.split('.').nth(1).unwrap_or("") {
            "health" | "hp" => (ConditionValue::Num(ctx.health as f32), 1),
            "maxhealth" | "maxhp" => (ConditionValue::Num(ctx.max_health as f32), 1),
            "dead" => (ConditionValue::Bool(ctx.dead), 1),
            "moving" => (ConditionValue::Bool(ctx.moving), 1),
            "facing" => (ConditionValue::Text(ctx.facing.clone()), 1),
            "gold" => (ConditionValue::Num(ctx.gold as f32), 1),
            "map" => (ConditionValue::Text(ctx.map.clone()), 1),
            "item" => {
                let count = ctx.items.get(args.get(1).map(String::as_str).unwrap_or(""))
                    .copied().unwrap_or(0);
                (ConditionValue::Num(count as f32), 2)
            }
            _ => return false,
        }
    } else {
        match kind.as_str() {
            "item" => {
                let count = ctx.items.get(args.get(1).map(String::as_str).unwrap_or(""))
                    .copied().unwrap_or(0);
                (ConditionValue::Num(count as f32), 2)
            }
            "gold" => (ConditionValue::Num(ctx.gold as f32), 1),
            "flag" => (ConditionValue::Bool(
                ctx.flags.contains(args.get(1).map(String::as_str).unwrap_or(""))
            ), 2),
            "map" => (ConditionValue::Text(ctx.map.clone()), 1),
            _ => return false,
        }
    };

    if args.len() <= offset {
        return value.truthy();
    }
    let token = args[offset].as_str();
    let is_operator = matches!(token, ">" | "<" | ">=" | "<=" | "==" | "!=");
    if is_operator {
        let right = args.get(offset + 1).cloned().unwrap_or_default();
        value.compare(token, &right)
    } else {
        value.compare("==", token)
    }
}

#[derive(Clone, Debug)]
enum ConditionValue {
    Num(f32),
    Text(String),
    Bool(bool),
}

impl ConditionValue {
    fn truthy(&self) -> bool {
        match self {
            ConditionValue::Num(v) => *v != 0.0,
            ConditionValue::Text(v) => !v.is_empty(),
            ConditionValue::Bool(v) => *v,
        }
    }

    fn compare(&self, op: &str, right: &str) -> bool {
        match self {
            ConditionValue::Num(left) => {
                let Some(rhs) = right.parse::<f32>().ok() else { return false };
                match op {
                    ">" => *left > rhs,
                    "<" => *left < rhs,
                    ">=" => *left >= rhs,
                    "<=" => *left <= rhs,
                    "==" => *left == rhs,
                    "!=" => *left != rhs,
                    _ => false,
                }
            }
            ConditionValue::Text(left) => match op {
                "==" => left == right,
                "!=" => left != right,
                _ => false,
            },
            ConditionValue::Bool(left) => match op {
                "==" => *left == (right == "true" || right == "1"),
                "!=" => *left != (right == "true" || right == "1"),
                _ => *left,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn tokenizes_and_parses_example() {
        let text = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/scripts/example.urpgs"),
        ).unwrap();
        let tokens = tokenize(&text);
        assert!(tokens.iter().any(|t| t == "if"));
        let mut parser = Parser { tokens, index: 0 };
        let nodes = parser.parse_block();
        assert!(nodes.len() >= 5);
    }

    #[test]
    fn evaluates_player_conditions() {
        let mut ctx = ScriptContext {
            health: 40,
            max_health: 100,
            dead: false,
            moving: false,
            facing: "right".into(),
            gold: 25,
            map: "map1".into(),
            flags: HashSet::new(),
            items: HashMap::new(),
        };
        assert!(eval_condition(&["player.health".into(), "<".into(), "50".into()], &ctx));
        assert!(eval_condition(&["player.facing".into(), "right".into()], &ctx));
        ctx.flags.insert("met_wizard".into());
        assert!(eval_condition(&["flag".into(), "met_wizard".into()], &ctx));
    }
}
