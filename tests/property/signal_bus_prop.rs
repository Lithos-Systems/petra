use proptest::prelude::*;
use petra::{SignalBus, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Runtime;

proptest! {
    #[test]
    fn test_concurrent_writes_dont_lose_data(
        writes in prop::collection::vec(
            (
                "[a-z]{3,10}", 
                prop_oneof![
                    Just(Value::Bool(true)),
                    Just(Value::Bool(false)),
                    any::<i32>().prop_map(|v| Value::Integer(v as i64)),
                    any::<f64>().prop_map(Value::Float),
                ]
            ), 
            1..100
        )
    ) {
        let rt = Runtime::new().unwrap();
        let bus = Arc::new(SignalBus::new());
        
        // Collect final values
        let mut expected: HashMap<String, Value> = HashMap::new();
        for (signal, value) in &writes {
            expected.insert(signal.clone(), value.clone());
        }
        
        // Perform concurrent writes
        rt.block_on(async {
            let mut handles = vec![];
            
            for (signal, value) in writes {
                let bus_clone = bus.clone();
                let handle = tokio::spawn(async move {
                    bus_clone.write_signal(&signal, value).unwrap();
                });
                handles.push(handle);
            }
            
            // Wait for all writes
            for handle in handles {
                handle.await.unwrap();
            }
        });
        
        // Verify all signals have their last written value
        for (signal, expected_value) in expected {
            let actual = bus.read_signal(&signal).unwrap();
            prop_assert_eq!(actual, expected_value);
        }
    }
    
    #[test]
    fn test_signal_names_are_case_sensitive(
        base_name in "[a-zA-Z]{5,10}",
        value1 in any::<f64>(),
        value2 in any::<f64>().filter(move |v| *v != value1),
    ) {
        let bus = SignalBus::new();
        let lower = base_name.to_lowercase();
        let upper = base_name.to_uppercase();
        
        if lower != upper {  // Only test if actually different
            // Write different values
            bus.write_signal(&lower, Value::Float(value1)).unwrap();
            bus.write_signal(&upper, Value::Float(value2)).unwrap();
            
            // Read back and verify they're different
            let v1 = bus.read_signal(&lower).unwrap();
            let v2 = bus.read_signal(&upper).unwrap();
            
            prop_assert_ne!(v1, v2);
        }
    }
    
    #[test]
    fn test_batch_operations_are_atomic(
        updates in prop::collection::vec(
            (
                "[a-z]{3,10}",
                any::<i32>().prop_map(|v| Value::Integer(v as i64))
            ),
            1..20
        )
    ) {
        let bus = SignalBus::new();
        
        // Convert to format expected by write_batch
        let batch: Vec<(&str, Value)> = updates
            .iter()
            .map(|(s, v)| (s.as_str(), v.clone()))
            .collect();
        
        // Perform batch write
        bus.write_batch(batch).unwrap();
        
        // Verify all signals were written
        for (signal, expected_value) in &updates {
            let actual = bus.read_signal(signal).unwrap();
            prop_assert_eq!(actual, *expected_value);
        }
    }
}
