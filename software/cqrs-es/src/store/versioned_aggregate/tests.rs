use crate::store::Version;
use crate::tests::test_aggregate::TestAggregate;

use super::VersionedAggregate;

#[test]
fn versioned_aggregate_serde() {
    let before = VersionedAggregate {
        version: Version(123),
        aggregate: TestAggregate(456),
    };
    let json = serde_json::to_string(&before).unwrap();
    println!("{}", json);
    let after: VersionedAggregate<TestAggregate> = serde_json::from_str(&json).unwrap();
    assert_eq!(before.version, after.version);
    assert_eq!(before.aggregate, after.aggregate);
}
