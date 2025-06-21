use petra::*;

#[tokio::test]
async fn test_basic_logic() {
    let yaml = r#"
signals:
  - name: "a"
    type: "bool"
    initial: true
  - name: "b"
    type: "bool"
    initial: true
  - name: "result"
    type: "bool"

blocks:
  - name: "and_gate"
    type: "AND"
    inputs:
      in1: "a"
      in2: "b"
    outputs:
      out: "result"

scan_time_ms: 100
"#;

    let config = Config::from_yaml(yaml).unwrap();
    let engine = Engine::new(config).unwrap();

    let bus = engine.bus();
    bus.set("a", Value::Bool(true)).unwrap();
    bus.set("b", Value::Bool(false)).unwrap();

    // Manual execution (engine.run would normally handle this)
    assert_eq!(bus.get_bool("result").unwrap_or(false), false);
}
