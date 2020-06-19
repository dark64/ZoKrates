//! Module containing static analysis
//!
//! @file mod.rs
//! @author Thibaut Schaeffer <thibaut@schaeff.fr>
//! @date 2018

mod constrain_inputs;
mod flat_propagation;
mod inline;
mod propagate_unroll;
mod propagation;
mod return_binder;
mod unconstrained_vars;
mod unroll;
mod statement_logger;

use self::constrain_inputs::InputConstrainer;
use self::inline::Inliner;
use self::propagate_unroll::PropagatedUnroller;
use self::propagation::Propagator;
use self::return_binder::ReturnBinder;
use self::unconstrained_vars::UnconstrainedVariableDetector;
use crate::flat_absy::FlatProg;
use crate::typed_absy::TypedProgram;
use ir::Prog;
use zokrates_field::Field;
use static_analysis::statement_logger::StatementLogger;

pub trait Analyse {
    fn analyse(self) -> Self;
}

impl<'ast, T: Field> Analyse for TypedProgram<'ast, T> {
    fn analyse(self) -> Self {
        let r = StatementLogger::create_logs(self);

        // propagated unrolling
        let r = PropagatedUnroller::unroll(r).unwrap_or_else(|e| panic!(e));

        // return binding
        let r = ReturnBinder::bind(r);
        // inline
        let r = Inliner::inline(r);
        // propagate
        let r = Propagator::propagate(r);
        // constrain inputs
        let r = InputConstrainer::constrain(r);

        r
    }
}

impl<T: Field> Analyse for FlatProg<T> {
    fn analyse(self) -> Self {
        self.propagate()
    }
}

impl<T: Field> Analyse for Prog<T> {
    fn analyse(self) -> Self {
        let r = UnconstrainedVariableDetector::detect(self);
        r
    }
}
