/// Integration test for forge_services
use forge_services::{IntoDomain, FromDomain};

/// A minimal concrete type to verify the traits are object-safe.
#[derive(Debug, Default)]
struct DummyDomain;

struct DummyExternal(String);

impl IntoDomain for DummyExternal {
    type Domain = DummyDomain;

    fn into_domain(self) -> Self::Domain {
        DummyDomain
    }
}

impl FromDomain<DummyDomain> for DummyExternal {
    fn from_domain(_value: DummyDomain) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Ok(DummyExternal(String::new()))
    }
}

#[test]
fn into_domain_trait_object_safe() {
    // Verify IntoDomain can be called and produces a Domain value
    let ext = DummyExternal("test".to_string());
    let _domain: DummyDomain = ext.into_domain();
    // Trait is object-safe — reached here without compilation errors
}

#[test]
fn from_domain_trait_object_safe() {
    // Verify FromDomain can be called and produces an external value
    let domain = DummyDomain;
    let ext: DummyExternal = FromDomain::from_domain(domain).expect("should not fail");
    assert_eq!(ext.0, "");
}
