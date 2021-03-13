use crate::macros::actions::Action;
use crate::macros::event_matching::{Event, EventMatcher};
use crate::match_checker::StringMatcher;
use crate::state::State;
use crate::macros::preconditions::Precondition;

pub mod actions;
pub mod event_matching;
pub mod preconditions;

#[derive(Clone, PartialEq, Debug)]
pub struct Scope {
    pub window_class: Option<StringMatcher>,
    pub window_name: Option<StringMatcher>,
}

impl Scope {
    pub fn new<'a>(
        window_class: Option<StringMatcher>,
        window_name: Option<StringMatcher>
    ) -> Scope {
        Scope { window_class, window_name }
    }
}

pub struct MacroBuilder {
    name: Option<String>,
    match_events: Vec<Box<EventMatcher>>,
    required_preconditions: Option<Vec<Precondition>>,
    actions: Vec<Action>,
    scope: Option<Scope>
}

impl <'a> MacroBuilder {
    pub fn from_event_matcher(
        event_matcher: Box<EventMatcher>
    ) -> MacroBuilder {
        MacroBuilder {
            name: None,
            match_events: vec![event_matcher],
            required_preconditions: None,
            actions: vec![],
            scope: None
        }
    }

    pub fn from_event_matchers(
        event_matchers: Vec<Box<EventMatcher>>
    ) -> MacroBuilder {
        MacroBuilder {
            name: None,
            match_events: event_matchers,
            required_preconditions: None,
            actions: vec![],
            scope: None
        }
    }

    pub fn set_event_matchers(mut self, event_matchers: Vec<Box<EventMatcher>>) -> Self {
        self.match_events = event_matchers;
        self
    }

    pub fn add_event_matcher(mut self, event_matcher: Box<EventMatcher>) -> Self {
        self.match_events.push(event_matcher);
        self
    }

    pub fn set_actions(mut self, actions: Vec<Action>) -> Self {
        self.actions = actions;
        self
    }

    pub fn add_action(mut self, action: Action) -> Self {
        self.actions.push(action);
        self
    }

    pub fn set_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn set_preconditions(mut self, preconditions: Vec<Precondition>) -> Self {
        self.required_preconditions = Some(preconditions);
        self
    }

    pub fn add_precondition(mut self, precondition: Precondition) -> Self {
        let mut new_preconditions : Vec<Precondition> = vec![];

        if let Some(_) = self.required_preconditions {
            let mut preconditions = self.required_preconditions.take().unwrap();
            new_preconditions.append(&mut preconditions);
            new_preconditions.push(precondition);
            self.required_preconditions = Some(new_preconditions);
        } else {
            self.required_preconditions = Some(vec![precondition]);
        }

        self
    }

    pub fn set_scope(mut self, scope: Scope) -> Self {
        self.scope = Some(scope);
        self
    }

    pub fn build(self) -> Macro {
        Macro {
            name: self.name,
            match_events: self.match_events,
            required_preconditions: self.required_preconditions,
            actions: self.actions,
            scope: self.scope
        }
    }
}

pub struct Macro {
    name: Option<String>,
    match_events: Vec<Box<EventMatcher>>,
    required_preconditions: Option<Vec<Precondition>>,
    actions: Vec<Action>,
    scope: Option<Scope>
}

impl Macro {
    pub fn name(&self) -> Option<&str> {
        if let Some(n) = &self.name {
            Some(n)
        } else {
            None
        }
    }

    /// Evaluates an incoming event, and it it matches against this macro's matching events,
    /// returns a list of actions to execute.
    pub fn evaluate<'b>(
        &self, event: &'b Event<'b>,
        state: &'b Box<dyn State>
    ) -> Option<&Vec<Action>> {

        // TODO: rejigger the order of these checks so the most expensive check is done last
        if !state.matches_scope(&self.scope) {
            return None
        }

        if let Some(conditions) = &self.required_preconditions {
            if conditions.iter().any(|condition| !state.matches(condition)) {
                return None;
            }
        }

        if self.matches_event(event, state) {
            Some(&self.actions)
        } else {
            None
        }
    }

    fn matches_event<'b>(&self, event: &Event<'b>, state: &'b Box<dyn State>) -> bool {
        self.match_events.iter().any(|event_matcher| {
            event_matcher.matches(event, state)
        })
    }
}
