//! Indexes for ontologies to enable faster searching

//! # Overview
//!

//! This module provides an interface and a number of multiplexers for
//! an `OntologyIndex`.

//! An indexed `MutableOntology` is one that uses one or more
//! `OntologyIndex` objects as the backing store for its
//! `AnnotatedAxiom`. These `AnnotatedAxiom` objects are shared
//! between different `OntologyIndex` objects using `Rc`. The
//! `OntologyIndex` interace does not provide any mechanisms for
//! searching or querying the index which need to be provided by
//! concrete implementations.

//! Indexes can be add to `OneIndexedOntology`, `TwoIndexedOntology`
//! and `ThreeIndexedOntology`, each of which operate something like a
//! named tuple, allowing differently typed `OntologyIndex` objects to
//! be added.
use crate::model::{AnnotatedAxiom, ArcStr, ForIRI, MutableOntology, Ontology, OntologyID, IRI, RcStr};
use std::borrow::Borrow;
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::Arc;

pub trait ForIndex<A: ForIRI>:
    Borrow<AnnotatedAxiom<A>>
    + Clone
    + Debug
    + Eq
    + From<AnnotatedAxiom<A>>
    + Hash
    + Ord
    + PartialEq
    + PartialOrd
{
    fn unwrap(&self) -> AnnotatedAxiom<A> {
        (*self.borrow()).clone()
    }
}

impl<A: ForIRI, T: ?Sized> ForIndex<A> for T where
    T: Borrow<AnnotatedAxiom<A>>
        + Clone
        + Debug
        + Eq
        + From<AnnotatedAxiom<A>>
        + Hash
        + Ord
        + PartialEq
        + PartialOrd
{
}

/// An `OntologyIndex` object.
///
/// The `OntologyIndex` stores references to an `AnnotatedAxiom` and
/// as they are added (or removed) from an `IndexedObject`. Objects
/// implementing this can provide search facilities over the
/// ``Ontology`; they should, in general, only provide search
/// facilities that they can do rapidly and quickly (e.g. in constant
/// or log time, not linear).
///
/// A given `OntologyIndex` object is not bound to keep references to
/// all `Rc<AnnotatedAxiom>` that are inserted into it, although at
/// least one `OntologyIndex` object for an `IndexedOntology` should
/// do, or the it will be dropped entirely. The `SetIndex` is a simple
/// way to achieving this.
pub trait OntologyIndex<A: ForIRI, AA: ForIndex<A>> {
    /// Potentially insert an AnnotatedAxiom to the index.
    ///
    /// If the index did not have this value present, true is returned.
    ///
    /// If the index did have this value present, false is returned.
    fn index_insert(&mut self, ax: AA) -> bool;

    /// Remove an AnnotatedAxiom from the index.
    ///
    /// If the index did have this value present, true is returned.
    ///
    /// If the index did not have this value present, false is returned.
    fn index_remove(&mut self, ax: &AnnotatedAxiom<A>) -> bool;

    fn index_take(&mut self, ax: &AnnotatedAxiom<A>) -> Option<AnnotatedAxiom<A>> {
        if self.index_remove(ax) {
            Some(ax.clone())
        } else {
            None
        }
    }
}

/// A NullOntologyIndex which does nothing.
#[derive(Default)]
pub struct NullIndex();
impl<A: ForIRI, AA: ForIndex<A>> OntologyIndex<A, AA> for NullIndex {
    /// Insert an item, always returns false
    fn index_insert(&mut self, _ax: AA) -> bool {
        false
    }

    /// Remove an item, always returns false
    fn index_remove(&mut self, _ax: &AnnotatedAxiom<A>) -> bool {
        false
    }
}

/// A `OneIndexedOntology` operates as a simple adaptor betweeen any
/// `OntologyIndex` and an `Ontology`.
#[derive(Default, Debug, Eq, PartialEq)]
pub struct OneIndexedOntology<A, AA, I>(I, OntologyID<A>, Option<IRI<A>>, PhantomData<AA>);

impl<A: ForIRI, AA: ForIndex<A>, I: Clone> Clone for OneIndexedOntology<A, AA, I> {
    fn clone(&self) -> Self {
        OneIndexedOntology(
            self.0.clone(),
            self.1.clone(),
            self.2.clone(),
            Default::default(),
        )
    }
}

impl<A: ForIRI, AA: ForIndex<A>, I: OntologyIndex<A, AA>> OneIndexedOntology<A, AA, I> {
    pub fn new(i: I) -> Self {
        OneIndexedOntology(
            i,
            Default::default(),
            Default::default(),
            Default::default(),
        )
    }

    pub fn i(&self) -> &I {
        &self.0
    }

    pub fn index(self) -> I {
        self.0
    }
}

impl<I> OneIndexedOntology<RcStr, Rc<AnnotatedAxiom<RcStr>>, I>
where
    I: OntologyIndex<RcStr, Rc<AnnotatedAxiom<RcStr>>>,
{
    pub fn new_rc(i: I) -> OneIndexedOntology<RcStr, Rc<AnnotatedAxiom<RcStr>>, I> {
        Self::new(i)
    }
}

impl<I> OneIndexedOntology<ArcStr, Arc<AnnotatedAxiom<ArcStr>>, I>
where
    I: OntologyIndex<ArcStr, Arc<AnnotatedAxiom<ArcStr>>>,
{
    pub fn new_arc(i: I) -> OneIndexedOntology<ArcStr, Arc<AnnotatedAxiom<ArcStr>>, I> {
        Self::new(i)
    }
}

impl<A: ForIRI, AA: ForIndex<A>, I: OntologyIndex<A, AA>> Ontology<A>
    for OneIndexedOntology<A, AA, I>
{
    fn id(&self) -> &OntologyID<A> {
        &self.1
    }

    fn mut_id(&mut self) -> &mut OntologyID<A> {
        &mut self.1
    }

    fn doc_iri(&self) -> &Option<IRI<A>> {
        &self.2
    }

    fn mut_doc_iri(&mut self) -> &mut Option<IRI<A>> {
        &mut self.2
    }
}

impl<A: ForIRI, AA: ForIndex<A>, I: OntologyIndex<A, AA>> MutableOntology<A>
    for OneIndexedOntology<A, AA, I>
{
    fn insert<IAA: Into<AnnotatedAxiom<A>>>(&mut self, ax: IAA) -> bool {
        let ax = ax.into();
        self.0.index_insert(ax.into())
    }

    fn take(&mut self, ax: &AnnotatedAxiom<A>) -> Option<AnnotatedAxiom<A>> {
        self.0.index_take(ax)
    }
}

/// A `TwoIndexOntology` implements `Ontology` and supports two
/// `OntologyIndex`. It itself implements `OntologyIndex` so that it
/// can be composed.
#[derive(Default, Debug, Eq, PartialEq)]
pub struct TwoIndexedOntology<
    A: ForIRI,
    AA: ForIndex<A>,
    I: OntologyIndex<A, AA>,
    J: OntologyIndex<A, AA>,
>(I, J, OntologyID<A>, Option<IRI<A>>, PhantomData<AA>);

impl<A: ForIRI, AA: ForIndex<A>, I: OntologyIndex<A, AA>, J: OntologyIndex<A, AA>>
    TwoIndexedOntology<A, AA, I, J>
{
    pub fn new(i: I, j: J, id: OntologyID<A>) -> Self {
        TwoIndexedOntology(i, j, id, Default::default(), Default::default())
    }

    pub fn i(&self) -> &I {
        &self.0
    }

    pub fn j(&self) -> &J {
        &self.1
    }

    pub fn index(self) -> (I, J) {
        (self.0, self.1)
    }
}

impl<A: ForIRI, AA: ForIndex<A>, I: OntologyIndex<A, AA>, J: OntologyIndex<A, AA>> Ontology<A>
    for TwoIndexedOntology<A, AA, I, J>
{
    fn id(&self) -> &OntologyID<A> {
        &self.2
    }

    fn mut_id(&mut self) -> &mut OntologyID<A> {
        &mut self.2
    }

    fn doc_iri(&self) -> &Option<IRI<A>> {
        &self.3
    }

    fn mut_doc_iri(&mut self) -> &mut Option<IRI<A>> {
        &mut self.3
    }
}

impl<A: ForIRI, AA: ForIndex<A>, I: OntologyIndex<A, AA>, J: OntologyIndex<A, AA>>
    MutableOntology<A> for TwoIndexedOntology<A, AA, I, J>
{
    fn insert<IAA: Into<AnnotatedAxiom<A>>>(&mut self, ax: IAA) -> bool {
        let ax = ax.into();
        self.index_insert(ax.into())
    }

    fn take(&mut self, ax: &AnnotatedAxiom<A>) -> Option<AnnotatedAxiom<A>> {
        self.index_take(ax)
    }
}

impl<A: ForIRI, AA: ForIndex<A>, I: OntologyIndex<A, AA>, J: OntologyIndex<A, AA>>
    OntologyIndex<A, AA> for TwoIndexedOntology<A, AA, I, J>
{
    fn index_insert(&mut self, ax: AA) -> bool {
        let rtn = self.0.index_insert(ax.clone());
        // Don't short circuit
        self.1.index_insert(ax) || rtn
    }

    fn index_remove(&mut self, ax: &AnnotatedAxiom<A>) -> bool {
        let rtn = self.0.index_remove(ax);
        // Don't short circuit
        self.1.index_remove(ax) || rtn
    }
}

/// ThreeIndexedOntology supports three indexes.
#[derive(Default, Debug)]
pub struct ThreeIndexedOntology<
    A: ForIRI,
    AA: ForIndex<A>,
    I: OntologyIndex<A, AA>,
    J: OntologyIndex<A, AA>,
    K: OntologyIndex<A, AA>,
>(TwoIndexedOntology<A, AA, I, TwoIndexedOntology<A, AA, J, K>>);

impl<
        A: ForIRI,
        AA: ForIndex<A>,
        I: OntologyIndex<A, AA>,
        J: OntologyIndex<A, AA>,
        K: OntologyIndex<A, AA>,
    > ThreeIndexedOntology<A, AA, I, J, K>
{
    pub fn new(i: I, j: J, k: K, id: OntologyID<A>) -> Self {
        ThreeIndexedOntology(TwoIndexedOntology(
            i,
            TwoIndexedOntology(
                j,
                k,
                Default::default(),
                Default::default(),
                Default::default(),
            ),
            id,
            Default::default(),
            Default::default(),
        ))
    }

    pub fn i(&self) -> &I {
        (&self.0).i()
    }

    pub fn j(&self) -> &J {
        (&self.0).j().i()
    }

    pub fn k(&self) -> &K {
        (self.0).j().j()
    }

    pub fn index(self) -> (I, J, K) {
        let index = (self.0).1.index();
        ((self.0).0, index.0, index.1)
    }
}

impl<
        A: ForIRI,
        AA: ForIndex<A>,
        I: OntologyIndex<A, AA>,
        J: OntologyIndex<A, AA>,
        K: OntologyIndex<A, AA>,
    > Ontology<A> for ThreeIndexedOntology<A, AA, I, J, K>
{
    fn id(&self) -> &OntologyID<A> {
        self.0.id()
    }

    fn mut_id(&mut self) -> &mut OntologyID<A> {
        self.0.mut_id()
    }

    fn doc_iri(&self) -> &Option<IRI<A>> {
        self.0.doc_iri()
    }

    fn mut_doc_iri(&mut self) -> &mut Option<IRI<A>> {
        self.0.mut_doc_iri()
    }
}

impl<
        A: ForIRI,
        AA: ForIndex<A>,
        I: OntologyIndex<A, AA>,
        J: OntologyIndex<A, AA>,
        K: OntologyIndex<A, AA>,
    > MutableOntology<A> for ThreeIndexedOntology<A, AA, I, J, K>
{
    fn insert<IAA: Into<AnnotatedAxiom<A>>>(&mut self, ax: IAA) -> bool {
        self.0.insert(ax)
    }

    fn take(&mut self, ax: &AnnotatedAxiom<A>) -> Option<AnnotatedAxiom<A>> {
        self.0.take(ax)
    }
}

impl<
        A: ForIRI,
        AA: ForIndex<A>,
        I: OntologyIndex<A, AA>,
        J: OntologyIndex<A, AA>,
        K: OntologyIndex<A, AA>,
    > OntologyIndex<A, AA> for ThreeIndexedOntology<A, AA, I, J, K>
{
    fn index_insert(&mut self, ax: AA) -> bool {
        let rtn = (self.0).0.index_insert(ax.clone());
        // Don't short cirtuit
        (self.0).1.index_insert(ax) || rtn
    }

    fn index_remove(&mut self, ax: &AnnotatedAxiom<A>) -> bool {
        let rtn = self.0.index_remove(ax);
        // Don't short circuit
        (self.0).1.index_remove(ax) || rtn
    }
}

/// FourIndexedOntology supports three indexes.
#[derive(Default, Debug)]
pub struct FourIndexedOntology<
    A: ForIRI,
    AA: ForIndex<A>,
    I: OntologyIndex<A, AA>,
    J: OntologyIndex<A, AA>,
    K: OntologyIndex<A, AA>,
    L: OntologyIndex<A, AA>,
>(TwoIndexedOntology<A, AA, I, ThreeIndexedOntology<A, AA, J, K, L>>);

impl<
        A: ForIRI,
        AA: ForIndex<A>,
        I: OntologyIndex<A, AA>,
        J: OntologyIndex<A, AA>,
        K: OntologyIndex<A, AA>,
        L: OntologyIndex<A, AA>,
    > FourIndexedOntology<A, AA, I, J, K, L>
{
    pub fn new(i: I, j: J, k: K, l: L, id: OntologyID<A>) -> Self {
        FourIndexedOntology(TwoIndexedOntology(
            i,
            ThreeIndexedOntology::new(j, k, l, Default::default()),
            id,
            Default::default(),
            Default::default(),
        ))
    }

    pub fn i(&self) -> &I {
        (&self.0).i()
    }

    pub fn j(&self) -> &J {
        (&self.0).j().i()
    }

    pub fn k(&self) -> &K {
        (self.0).j().j()
    }

    pub fn l(&self) -> &L {
        (self.0).j().k()
    }

    pub fn index(self) -> (I, J, K, L) {
        let index = (self.0).1.index();
        ((self.0).0, index.0, index.1, index.2)
    }
}

impl<
        A: ForIRI,
        AA: ForIndex<A>,
        I: OntologyIndex<A, AA>,
        J: OntologyIndex<A, AA>,
        K: OntologyIndex<A, AA>,
        L: OntologyIndex<A, AA>,
    > Ontology<A> for FourIndexedOntology<A, AA, I, J, K, L>
{
    fn id(&self) -> &OntologyID<A> {
        self.0.id()
    }

    fn mut_id(&mut self) -> &mut OntologyID<A> {
        self.0.mut_id()
    }

    fn doc_iri(&self) -> &Option<IRI<A>> {
        self.0.doc_iri()
    }

    fn mut_doc_iri(&mut self) -> &mut Option<IRI<A>> {
        self.0.mut_doc_iri()
    }
}

impl<
        A: ForIRI,
        AA: ForIndex<A>,
        I: OntologyIndex<A, AA>,
        J: OntologyIndex<A, AA>,
        K: OntologyIndex<A, AA>,
        L: OntologyIndex<A, AA>,
    > MutableOntology<A> for FourIndexedOntology<A, AA, I, J, K, L>
{
    fn insert<IAA: Into<AnnotatedAxiom<A>>>(&mut self, ax: IAA) -> bool {
        self.0.insert(ax)
    }

    fn take(&mut self, ax: &AnnotatedAxiom<A>) -> Option<AnnotatedAxiom<A>> {
        self.0.take(ax)
    }
}

#[cfg(test)]
mod test {

    use super::{
        FourIndexedOntology, NullIndex, OneIndexedOntology, ThreeIndexedOntology,
        TwoIndexedOntology,
    };
    use crate::{
        model::{AnnotatedAxiom, Build, MutableOntology, NamedEntity, RcStr},
        ontology::set::SetIndex,
    };

    fn stuff() -> (
        AnnotatedAxiom<RcStr>,
        AnnotatedAxiom<RcStr>,
        AnnotatedAxiom<RcStr>,
    ) {
        let b = Build::new_rc();
        let c: NamedEntity<_> = b.class("http://www.example.com/c").into();
        let o: NamedEntity<_> = b.object_property("http://www.example.com/p").into();
        let b: NamedEntity<_> = b.data_property("http://www.example.com/d").into();

        (c.into(), o.into(), b.into())
    }

    #[test]
    fn one_cons() {
        let _o = OneIndexedOntology::new_rc(SetIndex::new());
        assert!(true);
    }

    #[test]
    fn one_insert() {
        let mut o = OneIndexedOntology::new_rc(SetIndex::new());
        let e = stuff();
        o.insert(e.0);
        o.insert(e.1);
        o.insert(e.2);

        assert_eq!(o.i().into_iter().count(), 3);
    }

    #[test]
    fn one_remove() {
        let mut o = OneIndexedOntology::new_rc(SetIndex::new());
        let e = stuff();
        o.insert(e.0.clone());
        o.insert(e.1.clone());
        o.insert(e.2.clone());

        assert_eq!(o.i().into_iter().count(), 3);
        assert!(o.remove(&e.0));
        assert!(o.remove(&e.1));
        assert!(o.remove(&e.2));

        assert_eq!(o.i().into_iter().count(), 0);
        assert!(!o.remove(&e.0));
        assert!(!o.remove(&e.1));
        assert!(!o.remove(&e.2));
    }

    #[test]
    fn two_cons() {
        let _o = TwoIndexedOntology::new(SetIndex::new_rc(), SetIndex::new(), Default::default());
        assert!(true);

        let _o =
            TwoIndexedOntology::new(SetIndex::new_rc(), NullIndex::default(), Default::default());
        assert!(true);
    }

    #[test]
    fn two_insert() {
        let mut o =
            TwoIndexedOntology::new(SetIndex::new_rc(), SetIndex::new(), Default::default());
        let e = stuff();
        o.insert(e.0);
        o.insert(e.1);
        o.insert(e.2);

        assert_eq!(o.i().into_iter().count(), 3);
        assert_eq!(o.j().into_iter().count(), 3);
        assert_eq!(o.i(), o.j());
    }

    #[test]
    fn two_remove() {
        let mut o =
            TwoIndexedOntology::new(SetIndex::new_rc(), SetIndex::new(), Default::default());

        let e = stuff();
        o.insert(e.0.clone());
        o.insert(e.1.clone());
        o.insert(e.2.clone());

        assert_eq!(o.i().into_iter().count(), 3);
        assert!(o.remove(&e.0));
        assert!(o.remove(&e.1));
        assert!(o.remove(&e.2));

        assert_eq!(o.i().into_iter().count(), 0);
        assert!(!o.remove(&e.0));
        assert!(!o.remove(&e.1));
        assert!(!o.remove(&e.2));
        assert_eq!(o.i(), o.j());
    }

    #[test]
    fn three_remove() {
        let mut o = ThreeIndexedOntology::new(
            SetIndex::new_rc(),
            SetIndex::new(),
            SetIndex::new(),
            Default::default(),
        );

        let e = stuff();
        o.insert(e.0.clone());
        o.insert(e.1.clone());
        o.insert(e.2.clone());

        assert_eq!(o.i().into_iter().count(), 3);
        assert!(o.remove(&e.0));
        assert!(o.remove(&e.1));
        assert!(o.remove(&e.2));

        assert_eq!(o.i().into_iter().count(), 0);
        assert!(!o.remove(&e.0));
        assert!(!o.remove(&e.1));
        assert!(!o.remove(&e.2));

        assert_eq!(o.i(), o.j());
        assert_eq!(o.i(), o.k());
    }

    #[test]
    fn four_remove() {
        let mut o = FourIndexedOntology::new(
            SetIndex::new_rc(),
            SetIndex::new(),
            SetIndex::new(),
            SetIndex::new(),
            Default::default(),
        );

        let e = stuff();
        o.insert(e.0.clone());
        o.insert(e.1.clone());
        o.insert(e.2.clone());

        assert_eq!(o.i().into_iter().count(), 3);
        assert!(o.remove(&e.0));
        assert!(o.remove(&e.1));
        assert!(o.remove(&e.2));

        assert_eq!(o.i().into_iter().count(), 0);
        assert!(!o.remove(&e.0));
        assert!(!o.remove(&e.1));
        assert!(!o.remove(&e.2));

        assert_eq!(o.i(), o.j());
        assert_eq!(o.i(), o.k());
        assert_eq!(o.i(), o.l());
    }
}
