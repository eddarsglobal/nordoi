use nordoi_kernel::{ActionSpec, AtomicError, Capability, CapabilitySet, Effect};

#[test]
fn pure_action_cannot_use_undeclared_network_effect() {
    let action = ActionSpec::pure("calculate");
    let mut authority = CapabilitySet::new();

    // Even possessing authority is not enough:
    // an effect must also be declared by the action.
    authority.allow(Capability::Network("api.example.com".into()));

    let result = action.validate_effect(authority, &Effect::Network("api.example.com".into()));

    assert_eq!(
        result,
        Err(AtomicError::UndeclaredEffect(Effect::Network(
            "api.example.com".into()
        )))
    );
}

#[test]
fn declared_network_effect_without_capability_is_denied() {
    let action = ActionSpec::new("weather").declare(Effect::Network("weather.example".into()));

    let result = action.validate_effect(
        CapabilitySet::new(),
        &Effect::Network("weather.example".into()),
    );

    assert_eq!(
        result,
        Err(AtomicError::CapabilityDenied(Capability::Network(
            "weather.example".into()
        )))
    );
}

#[test]
fn declared_and_authorized_effect_is_allowed() {
    let effect = Effect::Network("weather.example".into());
    let capability = Capability::Network("weather.example".into());

    let action = ActionSpec::new("weather").declare(effect.clone());

    let mut authority = CapabilitySet::new();
    authority.allow(capability);

    assert!(action.validate_effect(authority, &effect).is_ok());
}

#[test]
fn authority_is_scope_exact() {
    let action = ActionSpec::new("weather")
        .declare(Effect::Network("weather.example".into()))
        .declare(Effect::Network("evil.example".into()));

    let mut authority = CapabilitySet::new();
    authority.allow(Capability::Network("weather.example".into()));

    assert!(action
        .validate_effect(
            authority.clone(),
            &Effect::Network("weather.example".into()),
        )
        .is_ok());

    assert_eq!(
        action.validate_effect(authority, &Effect::Network("evil.example".into()),),
        Err(AtomicError::CapabilityDenied(Capability::Network(
            "evil.example".into()
        )))
    );
}

#[test]
fn local_state_effect_needs_declaration_but_no_external_capability() {
    let effect = Effect::StateWrite;
    let action = ActionSpec::new("increment").declare(effect.clone());

    assert!(action
        .validate_effect(CapabilitySet::new(), &effect)
        .is_ok());
}
