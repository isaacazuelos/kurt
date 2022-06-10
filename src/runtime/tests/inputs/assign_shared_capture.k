let make_pair = (value) => {
    let getter = () => value;
    let setter = (new) => value = new;
    [getter, setter]
};

let list = make_pair(2);
let getter = list[0];
let setter = list[1];

let two = getter();
setter(7);
let seven = getter();
setter(11);

two + getter() + seven