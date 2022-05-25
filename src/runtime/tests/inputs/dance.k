let outer = () => {
    let value = 7;
    let middle = () => {
        let inner = () => {
            value
        };
        "crate inner closure";
        inner
    };
    "return from outer";
    middle
};
let mid = outer();
let inside = mid();
inside()