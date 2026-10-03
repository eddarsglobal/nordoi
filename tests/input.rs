use nordoi_kernel::{
    AtomicInputCore, AtomicKernel, InputAtomBridge, InputDeviceId, InputPayload, InputSelector,
    InputSignal, InputSource, InputTarget, PointerId, RenderNodeId, Value,
};

fn keyboard_device() -> InputDeviceId {
    InputDeviceId(1)
}

fn mouse_device() -> InputDeviceId {
    InputDeviceId(2)
}

fn gamepad_device() -> InputDeviceId {
    InputDeviceId(3)
}

#[test]
fn sequence_is_monotonic_and_deterministic() {
    let mut input = AtomicInputCore::new();

    let a = input
        .submit(
            InputSource::Keyboard,
            keyboard_device(),
            None,
            InputPayload::Key {
                code: 4,
                pressed: true,
                repeat: false,
            },
        )
        .unwrap();
    let b = input
        .submit(
            InputSource::Mouse,
            mouse_device(),
            None,
            InputPayload::PointerButton {
                pointer: PointerId(1),
                button: 0,
                pressed: true,
            },
        )
        .unwrap();

    assert_eq!(a.0, 1);
    assert_eq!(b.0, 2);
    let batch = input.drain();
    assert_eq!(batch.events[0].sequence.0, 1);
    assert_eq!(batch.events[1].sequence.0, 2);
}

#[test]
fn keyboard_without_explicit_target_routes_to_focus() {
    let mut input = AtomicInputCore::new();
    let focused = InputTarget::RenderNode(RenderNodeId(77));
    input.set_focus(Some(focused));

    input
        .submit(
            InputSource::Keyboard,
            keyboard_device(),
            None,
            InputPayload::Key {
                code: 13,
                pressed: true,
                repeat: false,
            },
        )
        .unwrap();

    assert_eq!(input.drain().events[0].target, focused);
}

#[test]
fn pointer_without_explicit_target_routes_global() {
    let mut input = AtomicInputCore::new();
    input.set_focus(Some(InputTarget::RenderNode(RenderNodeId(9))));

    input
        .submit(
            InputSource::Mouse,
            mouse_device(),
            None,
            InputPayload::PointerMove {
                pointer: PointerId(1),
                position: [10.0, 20.0],
                delta: [1.0, 2.0],
            },
        )
        .unwrap();

    assert_eq!(input.drain().events[0].target, InputTarget::Global);
}

#[test]
fn explicit_target_overrides_focus() {
    let mut input = AtomicInputCore::new();
    input.set_focus(Some(InputTarget::RenderNode(RenderNodeId(10))));
    let explicit = InputTarget::RenderNode(RenderNodeId(11));

    input
        .submit(
            InputSource::Keyboard,
            keyboard_device(),
            Some(explicit),
            InputPayload::Key {
                code: 9,
                pressed: true,
                repeat: false,
            },
        )
        .unwrap();

    assert_eq!(input.drain().events[0].target, explicit);
}

#[test]
fn focus_change_does_not_retarget_queued_event() {
    let mut input = AtomicInputCore::new();
    let first = InputTarget::RenderNode(RenderNodeId(1));
    let second = InputTarget::RenderNode(RenderNodeId(2));
    input.set_focus(Some(first));

    input
        .submit(
            InputSource::Keyboard,
            keyboard_device(),
            None,
            InputPayload::Key {
                code: 1,
                pressed: true,
                repeat: false,
            },
        )
        .unwrap();
    input.set_focus(Some(second));

    assert_eq!(input.drain().events[0].target, first);
}

#[test]
fn consecutive_pointer_moves_coalesce_to_latest_state() {
    let mut input = AtomicInputCore::new();
    for x in [1.0, 2.0, 3.0] {
        input
            .submit(
                InputSource::Mouse,
                mouse_device(),
                None,
                InputPayload::PointerMove {
                    pointer: PointerId(1),
                    position: [x, 4.0],
                    delta: [1.0, 0.0],
                },
            )
            .unwrap();
    }

    assert_eq!(input.pending_events(), 1);
    let event = &input.drain().events[0];
    assert_eq!(event.sequence.0, 3);
    assert_eq!(
        event.payload,
        InputPayload::PointerMove {
            pointer: PointerId(1),
            position: [3.0, 4.0],
            delta: [3.0, 0.0],
        }
    );
}

#[test]
fn pointer_button_is_a_coalescing_barrier() {
    let mut input = AtomicInputCore::new();
    input
        .submit(
            InputSource::Mouse,
            mouse_device(),
            None,
            InputPayload::PointerMove {
                pointer: PointerId(1),
                position: [1.0, 1.0],
                delta: [1.0, 1.0],
            },
        )
        .unwrap();
    input
        .submit(
            InputSource::Mouse,
            mouse_device(),
            None,
            InputPayload::PointerButton {
                pointer: PointerId(1),
                button: 0,
                pressed: true,
            },
        )
        .unwrap();
    input
        .submit(
            InputSource::Mouse,
            mouse_device(),
            None,
            InputPayload::PointerMove {
                pointer: PointerId(1),
                position: [2.0, 2.0],
                delta: [1.0, 1.0],
            },
        )
        .unwrap();

    assert_eq!(input.drain().len(), 3);
}

#[test]
fn axis_state_coalesces_but_buttons_do_not() {
    let mut input = AtomicInputCore::new();
    for value in [0.1, 0.5, 1.0] {
        input
            .submit(
                InputSource::Gamepad,
                gamepad_device(),
                None,
                InputPayload::Axis { axis: 2, value },
            )
            .unwrap();
    }
    assert_eq!(input.pending_events(), 1);

    input
        .submit(
            InputSource::Gamepad,
            gamepad_device(),
            None,
            InputPayload::Button {
                button: 7,
                pressed: true,
            },
        )
        .unwrap();
    input
        .submit(
            InputSource::Gamepad,
            gamepad_device(),
            None,
            InputPayload::Button {
                button: 7,
                pressed: false,
            },
        )
        .unwrap();

    assert_eq!(input.pending_events(), 3);
}

#[test]
fn non_finite_pointer_input_is_rejected_without_queue_work() {
    let mut input = AtomicInputCore::new();
    let result = input.submit(
        InputSource::Mouse,
        mouse_device(),
        None,
        InputPayload::PointerMove {
            pointer: PointerId(1),
            position: [f32::NAN, 0.0],
            delta: [0.0, 0.0],
        },
    );

    assert!(result.is_err());
    assert_eq!(input.pending_events(), 0);
}

#[test]
fn axis_outside_normalized_range_is_rejected() {
    let mut input = AtomicInputCore::new();
    let result = input.submit(
        InputSource::Gamepad,
        gamepad_device(),
        None,
        InputPayload::Axis {
            axis: 0,
            value: 1.01,
        },
    );

    assert!(result.is_err());
    assert_eq!(input.pending_events(), 0);
}

#[test]
fn xr_pose_orientation_is_normalized_and_zero_is_rejected() {
    let mut input = AtomicInputCore::new();
    assert!(input
        .submit(
            InputSource::XrController,
            InputDeviceId(8),
            None,
            InputPayload::Pose {
                position: [0.0, 1.0, 2.0],
                orientation: [0.0, 0.0, 0.0, 0.0],
            },
        )
        .is_err());

    input
        .submit(
            InputSource::XrController,
            InputDeviceId(8),
            None,
            InputPayload::Pose {
                position: [0.0, 1.0, 2.0],
                orientation: [0.0, 0.0, 0.0, -2.0],
            },
        )
        .unwrap();

    let payload = &input.drain().events[0].payload;
    assert_eq!(
        payload,
        &InputPayload::Pose {
            position: [0.0, 1.0, 2.0],
            orientation: [0.0, 0.0, 0.0, 1.0],
        }
    );
}

#[test]
fn negative_zero_is_canonicalized() {
    let mut input = AtomicInputCore::new();
    input
        .submit(
            InputSource::Mouse,
            mouse_device(),
            None,
            InputPayload::Scroll { delta: [-0.0, 0.0] },
        )
        .unwrap();

    match input.drain().events[0].payload {
        InputPayload::Scroll { delta } => {
            assert!(!delta[0].is_sign_negative());
            assert!(!delta[1].is_sign_negative());
        }
        _ => panic!("expected scroll input"),
    }
}

#[test]
fn input_bridge_commits_one_atomic_batch_into_nam() {
    let mut kernel = AtomicKernel::new();
    let root = kernel.root_domain();
    let key_atom = kernel.create_atom(false);
    let axis_atom = kernel.create_atom(0.0_f64);

    let mut bridge = InputAtomBridge::new(root);
    bridge
        .bind(
            &kernel,
            InputSelector::any(InputSignal::KeyPressed { code: 42 }),
            key_atom,
        )
        .unwrap();
    bridge
        .bind(
            &kernel,
            InputSelector::any(InputSignal::Axis { axis: 1 }),
            axis_atom,
        )
        .unwrap();

    let mut input = AtomicInputCore::new();
    input
        .submit(
            InputSource::Keyboard,
            keyboard_device(),
            None,
            InputPayload::Key {
                code: 42,
                pressed: true,
                repeat: false,
            },
        )
        .unwrap();
    input
        .submit(
            InputSource::Gamepad,
            gamepad_device(),
            None,
            InputPayload::Axis {
                axis: 1,
                value: 0.75,
            },
        )
        .unwrap();

    let report = bridge.apply_batch(&mut kernel, &input.drain()).unwrap();
    assert_eq!(report.events, 2);
    assert_eq!(report.matched_bindings, 2);
    assert_eq!(report.changed_atoms, 2);
    assert_eq!(kernel.get(key_atom).unwrap(), &Value::Bool(true));
    assert_eq!(kernel.get(axis_atom).unwrap(), &Value::Float(0.75));
}

#[test]
fn multiple_state_transitions_to_same_atom_collapse_to_final_snapshot() {
    let mut kernel = AtomicKernel::new();
    let atom = kernel.create_atom(Value::Null);
    let mut bridge = InputAtomBridge::new(kernel.root_domain());
    bridge
        .bind(
            &kernel,
            InputSelector::any(InputSignal::KeyPressed { code: 7 }),
            atom,
        )
        .unwrap();

    let mut input = AtomicInputCore::new();
    for pressed in [true, false] {
        input
            .submit(
                InputSource::Keyboard,
                keyboard_device(),
                None,
                InputPayload::Key {
                    code: 7,
                    pressed,
                    repeat: false,
                },
            )
            .unwrap();
    }

    let report = bridge.apply_batch(&mut kernel, &input.drain()).unwrap();
    assert_eq!(report.matched_bindings, 2);
    assert_eq!(report.staged_writes, 1);
    assert_eq!(report.changed_atoms, 1);
    assert_eq!(kernel.get(atom).unwrap(), &Value::Bool(false));
}

#[test]
fn repeated_identical_input_value_creates_zero_new_state_work() {
    let mut kernel = AtomicKernel::new();
    let atom = kernel.create_atom(false);
    let mut bridge = InputAtomBridge::new(kernel.root_domain());
    bridge
        .bind(
            &kernel,
            InputSelector::any(InputSignal::KeyPressed { code: 5 }),
            atom,
        )
        .unwrap();

    let mut input = AtomicInputCore::new();
    for _ in 0..2 {
        input
            .submit(
                InputSource::Keyboard,
                keyboard_device(),
                None,
                InputPayload::Key {
                    code: 5,
                    pressed: true,
                    repeat: false,
                },
            )
            .unwrap();
    }
    let first = bridge.apply_batch(&mut kernel, &input.drain()).unwrap();
    assert_eq!(first.changed_atoms, 1);
    kernel.flush();

    input
        .submit(
            InputSource::Keyboard,
            keyboard_device(),
            None,
            InputPayload::Key {
                code: 5,
                pressed: true,
                repeat: true,
            },
        )
        .unwrap();
    let second = bridge.apply_batch(&mut kernel, &input.drain()).unwrap();
    assert_eq!(second.changed_atoms, 0);
    assert_eq!(second.scheduled_atoms, 0);
    assert_eq!(kernel.pending_work(), 0);
}

#[test]
fn bridge_binding_respects_ownership_boundary() {
    let mut kernel = AtomicKernel::new();
    let foreign = kernel.create_domain("foreign").unwrap();
    let atom = kernel.create_atom_owned(foreign, false).unwrap();
    let mut bridge = InputAtomBridge::new(kernel.root_domain());

    assert!(bridge
        .bind(
            &kernel,
            InputSelector::any(InputSignal::KeyPressed { code: 1 }),
            atom,
        )
        .is_err());
    assert_eq!(kernel.get(atom).unwrap(), &Value::Bool(false));
}

#[test]
fn selector_can_scope_source_device_and_target() {
    let mut kernel = AtomicKernel::new();
    let atom = kernel.create_atom(0.0_f64);
    let target = InputTarget::RenderNode(RenderNodeId(99));
    let mut bridge = InputAtomBridge::new(kernel.root_domain());
    bridge
        .bind(
            &kernel,
            InputSelector {
                source: Some(InputSource::Gamepad),
                device: Some(gamepad_device()),
                target: Some(target),
                signal: InputSignal::Axis { axis: 0 },
            },
            atom,
        )
        .unwrap();

    let mut input = AtomicInputCore::new();
    input
        .submit(
            InputSource::Gamepad,
            InputDeviceId(44),
            Some(target),
            InputPayload::Axis {
                axis: 0,
                value: 0.5,
            },
        )
        .unwrap();
    input
        .submit(
            InputSource::Gamepad,
            gamepad_device(),
            Some(target),
            InputPayload::Axis {
                axis: 0,
                value: 0.8,
            },
        )
        .unwrap();

    let report = bridge.apply_batch(&mut kernel, &input.drain()).unwrap();
    assert_eq!(report.matched_bindings, 1);
    assert_eq!(kernel.get(atom).unwrap(), &Value::Float(f64::from(0.8_f32)));
}

#[test]
fn no_matching_binding_creates_zero_kernel_work() {
    let mut kernel = AtomicKernel::new();
    let atom = kernel.create_atom(false);
    let mut bridge = InputAtomBridge::new(kernel.root_domain());
    bridge
        .bind(
            &kernel,
            InputSelector::any(InputSignal::KeyPressed { code: 1 }),
            atom,
        )
        .unwrap();

    let mut input = AtomicInputCore::new();
    input
        .submit(
            InputSource::Keyboard,
            keyboard_device(),
            None,
            InputPayload::Key {
                code: 2,
                pressed: true,
                repeat: false,
            },
        )
        .unwrap();

    let report = bridge.apply_batch(&mut kernel, &input.drain()).unwrap();
    assert_eq!(report.matched_bindings, 0);
    assert_eq!(report.staged_writes, 0);
    assert_eq!(report.changed_atoms, 0);
    assert_eq!(kernel.pending_work(), 0);
}
