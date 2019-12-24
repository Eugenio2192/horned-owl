use self::Namespace::*;
use enum_meta::*;

use crate::model::Build;
use crate::model::Facet;
use crate::model::NamedEntity;

use failure::Error;

pub trait WithIRI<'a>: Meta<&'a IRIString> {
    /// Return a string representation of the IRI associated with this
    /// entity.
    fn iri_s(&self) -> &'a String {
        &self.meta().0
    }

    fn iri_b(&self) -> &'a [u8] {
        &self.meta().0.as_bytes()
    }

    fn iri_str(&self) -> &'a str {
        &self.meta().0[..]
    }

    fn var_s(tag: &'a str) -> Option<Self> {
        Self::var_b(tag.as_bytes())
    }

    fn var_b(tag: &'a [u8]) -> Option<Self> {
        for v in Self::all() {
            if tag == v.iri_b() {
                return Some(v);
            }
        }
        None
    }
}

pub struct IRIString(String);

impl<'a, T> WithIRI<'a> for T where T: Meta<&'a IRIString> {}

fn to_meta(s: &str) -> IRIString {
    IRIString(s.to_string())
}

fn extend<'a, I>(i: I, s: &'a str) -> IRIString
where
    I: WithIRI<'a>,
{
    to_meta(&format!("{}{}", i.iri_s(), s))
}

#[derive(Debug, Eq, PartialEq)]
pub enum Namespace {
    OWL,
    RDF,
    RDFS,
    XSD,
}

lazy_meta! {
    Namespace, IRIString, METANS;
    OWL, to_meta("http://www.w3.org/2002/07/owl#");
    RDF, to_meta("http://www.w3.org/1999/02/22-rdf-syntax-ns#");
    RDFS, to_meta("http://www.w3.org/2000/01/rdf-schema#");
    XSD, to_meta("http://www.w3.org/2001/XMLSchema#");
}

pub enum RDF {
    First,
    Nil,
    Rest,
    Type,
}

lazy_meta! {
    RDF, IRIString, METARDF;
    First, extend(RDF, "first");
    Nil, extend(RDF, "nil");
    Rest, extend(RDF, "rest");
    Type, extend(RDF, "type");
}

pub enum RDFS {
    SubClassOf,
}

lazy_meta! {
    RDFS, IRIString, METARDFS;
    SubClassOf, extend(RDFS, "subClassOf");
}

pub enum OWL {
    // Lower case
    AllValuesFrom,
    AnnotatedSource,
    AnnotatedProperty,
    AnnotatedTarget,
    IntersectionOf,
    OnProperty,
    SomeValuesFrom,

    // Upper Case
    Axiom,
    Class,
    Datatype,
    DatatypeProperty,
    Nothing,
    ObjectProperty,
    Ontology,
    Restriction,
    Thing,
    VersionIRI,
}

lazy_meta! {
    OWL, IRIString, METAOWL;

    // Lower Case
    AllValuesFrom, extend(OWL, "allValuesFrom");
    AnnotatedSource, extend(OWL, "annotatedSource");
    AnnotatedProperty, extend(OWL, "annotatedProperty");
    AnnotatedTarget, extend(OWL, "annotatedTarget");
    IntersectionOf, extend(OWL, "intersectionOf");
    OnProperty, extend(OWL, "onProperty");
    SomeValuesFrom, extend(OWL, "someValuesFrom");

    // Upper Case
    Axiom, extend(OWL, "Axiom");
    Class, extend(OWL, "Class");
    Datatype, extend(OWL, "Datatype");
    DatatypeProperty, extend(OWL, "DatatypeProperty");
    Nothing, extend(OWL, "Nothing");
    ObjectProperty, extend(OWL, "ObjectProperty");
    Ontology, extend(OWL, "Ontology");
    Restriction, extend(OWL, "Restriction");
    Thing, extend(OWL, "Thing");
    VersionIRI, extend(OWL, "versionIRI");
}

#[test]
fn meta_testing() {
    assert_eq!("http://www.w3.org/2002/07/owl#", OWL.iri_s());
    assert_eq!(b"http://www.w3.org/2002/07/owl#", OWL.iri_b());

    assert_eq!(
        Namespace::var_s("http://www.w3.org/2002/07/owl#").unwrap(),
        OWL
    );

    assert_eq!(
        Namespace::var_b(b"http://www.w3.org/2002/07/owl#").unwrap(),
        OWL
    );
}

pub fn entity_for_iri(type_iri: &str, entity_iri: &str, b: &Build) -> Result<NamedEntity,Error> {
    // Datatypes are handled here because they are not a
    // "type" but an "RDF schema" element.
    if type_iri == "http://www.w3.org/2000/01/rdf-schema#Datatype" {
        return Ok(b.datatype(entity_iri).into());
    }

    if type_iri.len() < 30  {
        bail!("IRI is not for a type of entity:{}", type_iri);
    }

    Ok(
        match &type_iri[30..] {
            "Class" => b.class(entity_iri).into(),
            "ObjectProperty" => b.object_property(entity_iri).into(),
            "DatatypeProperty" => b.data_property(entity_iri).into(),
            "AnnotationProperty" => b.annotation_property(entity_iri).into(),
            "NamedIndividual" => b.named_individual(entity_iri).into(),
            _ => bail!("IRI is not a type of entity:{}", type_iri),
        })
}

#[test]
pub fn test_entity_for_iri() {
    let b = Build::new();

    assert!(entity_for_iri("http://www.w3.org/2002/07/owl#Class",
                           "http://www.example.com", &b).is_ok());
    assert!(entity_for_iri("http://www.w3.org/2002/07/owl#Fred",
                                 "http://www.example.com", &b).is_err());
}

pub enum OWL2Datatype {
    RDFSLiteral,
}

lazy_meta! {
    OWL2Datatype, IRIString, METAOWL2DATATYPE;
    RDFSLiteral, extend(RDFS, "Literal")
}

pub enum AnnotationBuiltIn {
    LABEL,
    COMMENT,
    SEEALSO,
    ISDEFINEDBY,
    DEPRECATED,
    VERSIOINFO,
    PRIORVERSION,
    BACKWARDCOMPATIBLEWITH,
    INCOMPATIBLEWITH,
}

lazy_meta! {
    AnnotationBuiltIn, IRIString, METAANNOTATIONBUILTIN;
    LABEL, extend(RDFS, "label");
    COMMENT, extend(RDFS, "comment");
    SEEALSO, extend(RDFS, "seeAlso");
    ISDEFINEDBY, extend(RDFS, "isDefinedBy");
    DEPRECATED, extend(OWL, "deprecated");
    VERSIOINFO, extend(OWL, "versionInfo");
    PRIORVERSION, extend(OWL, "priorVersion");
    BACKWARDCOMPATIBLEWITH, extend(OWL, "backwardCompatibleWith");
    INCOMPATIBLEWITH, extend(OWL, "incompatibleWith");
}

pub fn is_annotation_builtin(iri: &String) -> bool {
    for meta in AnnotationBuiltIn::all() {
        if meta.iri_s() == iri {
            return true;
        }
    }
    return false;
}

#[test]
fn annotation_builtin(){
    assert!(is_annotation_builtin(&"http://www.w3.org/2002/07/owl#deprecated".to_string()));
    assert!(is_annotation_builtin(&"http://www.w3.org/2000/01/rdf-schema#comment".to_string()));
    assert!(!is_annotation_builtin(&"http://www.w3.org/2002/07/owl#fred".to_string()));
}

lazy_meta!{
    Facet, IRIString, METAFACET;
    Length, extend(XSD, "length");
    MinLength, extend(XSD, "minLength");
    MaxLength, extend(XSD, "maxLength");
    Pattern, extend(XSD, "pattern");
    MinInclusive, extend(XSD, "minInclusive");
    MinExclusive, extend(XSD, "minExclusive");
    MaxInclusive, extend(XSD, "maxInclusive");
    MaxExclusive, extend(XSD, "maxExclusive");
    TotalDigits, extend(XSD, "totalDigits");
    FractionDigits, extend(XSD, "fractionDigits");
    LangRange, extend(RDF, "langRange");
}

#[test]
fn facet_meta() {
    assert_eq!(
        Facet::MinLength.iri_s(),
        "http://www.w3.org/2001/XMLSchema#minLength"
    );

    assert_eq!(
        Facet::Pattern.iri_s(),
        "http://www.w3.org/2001/XMLSchema#pattern"
    );

    assert_eq!(
        Facet::var_s(&"http://www.w3.org/2001/XMLSchema#pattern".to_string()).unwrap(),
        Facet::Pattern
    );

    assert_eq!(
        Facet::var_b(b"http://www.w3.org/2001/XMLSchema#minExclusive").unwrap(),
        Facet::MinExclusive
    );
}
