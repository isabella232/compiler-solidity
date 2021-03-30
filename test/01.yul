{
    function bar() -> x, y {
        x := 42
        y := 25
    }

    function foo() {
        let x := 1
        let y := 2
        x, y := bar()
    }
}
