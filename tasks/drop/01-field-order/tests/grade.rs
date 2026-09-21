//! The grader. It measures reality and reports it; it asserts no expected order,
//! because a predict task stores no answer.

#[test]
fn report_the_drop_order() {
    let order = task_drop_01_field_order::observe();
    assert_eq!(order.len(), 5, "all five tokens must drop exactly once");
    underust_grade::emit("order", &order);
}
