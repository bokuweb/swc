use super::*;
use crate::{
    compat::{
        es2015::{block_scoping, function_name, Classes},
        es3::ReservedWord,
    },
    resolver,
};
use swc_ecma_parser::{EsConfig, Syntax};

fn syntax() -> Syntax {
    Syntax::Es(EsConfig {
        class_private_props: true,
        class_props: true,
        ..Default::default()
    })
}

fn tr() -> impl Fold<Module> {
    chain!(
        resolver(),
        function_name(),
        class_properties(),
        Classes,
        block_scoping(),
        ReservedWord,
    )
}

test!(
    syntax(),
    |_| tr(),
    public_static_infer_name,
    r#"
var Foo = class {
  static num = 0;
}

"#,
    r#"
var Foo = function() {
    var Foo = function Foo() {
        _classCallCheck(this, Foo);
    };
    _defineProperty(Foo, 'num', 0);
    return Foo;
}();
"#
);

test_exec!(
    syntax(),
    |_| tr(),
    public_call_exec,
    r#"
class Foo {
  foo = function() {
    return this;
  }

  test(other) {
    return [this.foo(), other.foo()];
  }
}

const f = new Foo;
const o = new Foo;
const test = f.test(o);
expect(test[0]).toBe(f);
expect(test[1]).toBe(o);

"#
);

test!(
    syntax(),
    |_| tr(),
    public_instance_computed,
    r#"
function test(x) {
  class F {
    [x] = 1;
    constructor() {}
  }

  x = 'deadbeef';
  expect(new F().foo).toBe(1);
  x = 'wrong';
  expect(new F().foo).toBe(1);
}

test('foo');

"#,
    r#"
function test(x) {
  var _x = x;

  var F = function F() {
    

    _classCallCheck(this, F);
    _defineProperty(this, _x, 1);
  };

  x = 'deadbeef';
  expect(new F().foo).toBe(1);
  x = 'wrong';
  expect(new F().foo).toBe(1);
}

test('foo');

"#
);

test!(
    syntax(),
    |_| tr(),
    public_super_statement,
    r#"
class Foo extends Bar {
  bar = "foo";

  constructor() {
    super();
  }
}

"#,
    r#"
var Foo =
/*#__PURE__*/
function (_Bar) {
  

  _inherits(Foo, _Bar);

  function Foo() {
    var _this;

    _classCallCheck(this, Foo);
    _this = _possibleConstructorReturn(this, _getPrototypeOf(Foo).call(this));
    _defineProperty(_assertThisInitialized(_this), "bar", "foo");
    return _this;
  }

  return Foo;
}(Bar);

"#
);

test!(
    syntax(),
    |_| tr(),
    private_foobar,
    r#"
class Child extends Parent {
  constructor() {
    super();
  }

  #scopedFunctionWithThis = () => {
    this.name = {};
  }
}

"#,
    r#"
var Child =
/*#__PURE__*/
function (_Parent) {
  

  _inherits(Child, _Parent);

  function Child() {
    var _this;

    _classCallCheck(this, Child);
    _this = _possibleConstructorReturn(this, _getPrototypeOf(Child).call(this));

    _scopedFunctionWithThis.set(_assertThisInitialized(_this), {
      writable: true,
      value: () => {
        _this.name = {};
      }
    });

    return _this;
  }

  return Child;
}(Parent);

var _scopedFunctionWithThis = new WeakMap();

"#
);

test_exec!(
    syntax(),
    |_| tr(),
    private_call_exec,
    r#"
class Foo {
  #foo = function() {
    return this;
  }

  test(other) {
    return [this.#foo(), other.#foo()];
  }
}

const f = new Foo;
const o = new Foo;
const test = f.test(o);
expect(test[0]).toBe(f);
expect(test[1]).toBe(o);

"#
);

test!(
    syntax(),
    |_| tr(),
    public_derived_multiple_supers,
    r#"
class Foo extends Bar {
  bar = "foo";

  constructor() {
    if (condition) {
      super();
    } else {
      super();
    }
  }
}

"#,
    r#"
var Foo =
/*#__PURE__*/
function (_Bar) {
  

  _inherits(Foo, _Bar);

  function Foo() {
    var _this;

    _classCallCheck(this, Foo);

    if (condition) {
      _this = _possibleConstructorReturn(this, _getPrototypeOf(Foo).call(this));
      _defineProperty(_assertThisInitialized(_this), "bar", "foo");
    } else {
      _this = _possibleConstructorReturn(this, _getPrototypeOf(Foo).call(this));
      _defineProperty(_assertThisInitialized(_this), "bar", "foo");
    }

    return _possibleConstructorReturn(_this);
  }

  return Foo;
}(Bar);

"#
);

test_exec!(
    syntax(),
    |_| tr(),
    private_static_call_exec,
    r#"
class Foo {
  static #foo = function(x) {
    return x;
  }

  test(x) {
    return Foo.#foo(x);
  }
}

const f = new Foo;
const test = f.test();
expect(f.test("bar")).toBe("bar");

"#
);

test_exec!(
    syntax(),
    |_| tr(),
    private_instance_undefined_exec,
    r#"
class Foo {
  #bar;

  test() {
    return this.#bar;
  }
}

expect(new Foo().test()).toBe(undefined);

"#
);

test_exec!(
    syntax(),
    |_| tr(),
    private_instance_exec,
    r#"
class Foo {
  #bar = "foo";

  test() {
    return this.#bar;
  }

  update() {
    this.#bar++;
  }

  set(val) {
    this.#bar = val;
  }

  static test(foo) {
    return foo.#bar;
  }

  static update(foo) {
    foo.#bar **= 2;
  }
}

const f = new Foo();
expect(f.test()).toBe("foo");
expect(Foo.test(f)).toBe("foo");
expect("bar" in f).toBe(false);

f.set(1);
expect(f.test()).toBe(1);
f.update();
expect(Foo.test(f)).toBe(2);
Foo.update(f);
expect(f.test()).toBe(4);

"#
);

test!(
    syntax(),
    |_| tr(),
    public_regression_t6719,
    r#"
function withContext(ComposedComponent) {
    return class WithContext extends Component {

        static propTypes = {
            context: PropTypes.shape(
                {
                    addCss: PropTypes.func,
                    setTitle: PropTypes.func,
                    setMeta: PropTypes.func,
                }
            ),
        };
    };
}

"#,
    r#"
function withContext(ComposedComponent) {
  return (function() {
    var WithContext = function(_Component) {
      _inherits(WithContext, _Component);
      function WithContext() {
        _classCallCheck(this, WithContext);
        return _possibleConstructorReturn(this, _getPrototypeOf(WithContext).apply(this, arguments));
      }
      return WithContext;
    }(Component);
    _defineProperty(WithContext, 'propTypes', {
      context: PropTypes.shape({
        addCss: PropTypes.func,
        setTitle: PropTypes.func,
        setMeta: PropTypes.func 
      }) 
    });
    return WithContext;
  })();
}

"#
);

test!(
    syntax(),
    |_| tr(),
    public_super_with_collision,
    r#"
class A {
  force = force;
  foo = super.method();

  constructor(force) {}
}

"#,
    r#"
var A = function A(force1) {
  

  _classCallCheck(this, A);
  _defineProperty(this, "force", force);
  _defineProperty(this, "foo", _get(_getPrototypeOf(A.prototype), "method", this).call(this));
};

"#
);

test!(
    syntax(),
    |_| tr(),
    public_call,
    r#"
class Foo {
  foo = function() {
    return this;
  }

  test(other) {
    this.foo();
    other.obj.foo();
  }
}

"#,
    r#"
var Foo =
/*#__PURE__*/
function () {
  

  function Foo() {
    _classCallCheck(this, Foo);
    _defineProperty(this, "foo", function () {
      return this;
    });
  }

  _createClass(Foo, [{
    key: "test",
    value: function test(other) {
      this.foo();
      other.obj.foo();
    }
  }]);
  return Foo;
}();

"#
);

test_exec!(
    syntax(),
    |_| tr(),
    public_instance_computed_exec,
    r#"
function test(x) {
  class F {
    [x] = 1;
    constructor() {}
  }

  x = 'deadbeef';
  expect(new F().foo).toBe(1);
  x = 'wrong';
  expect(new F().foo).toBe(1);
}

test('foo');

"#
);

test!(
    syntax(),
    |_| tr(),
    private_declaration_order,
    r#"
class C {
  y = this.#x;
  #x;
}

expect(() => {
  new C();
}).toThrow();

"#,
    r#"
var C = function C() {
  

  _classCallCheck(this, C);
  _defineProperty(this, "y", _classPrivateFieldGet(this, _x));

  _x.set(this, {
    writable: true,
    value: void 0
  });
};

var _x = new WeakMap();

expect(() => {
  new C();
}).toThrow();

"#
);

test!(
    syntax(),
    |_| tr(),
    nested_class_super_call_in_key,
    r#"

class Hello {
  constructor() {
    return {
      toString() {
        return 'hello';
      },
    };
  }
}

class Outer extends Hello {
  constructor() {
    class Inner {
      [super()] = "hello";
    }

    return new Inner();
  }
}

expect(new Outer().hello).toBe('hello');

"#,
    r#"


var Hello = function Hello() {
  _classCallCheck(this, Hello);
  return {
    toString() {
      return 'hello';
    }

  };
};

var Outer = function (_Hello) {
  _inherits(Outer, _Hello);

  function Outer() {
    var _this;

    _classCallCheck(this, Outer);

    var _ref = _this = _possibleConstructorReturn(this, _getPrototypeOf(Outer).call(this));

    var Inner = function Inner() {
      _classCallCheck(this, Inner);
      _defineProperty(this, _ref, "hello");
    };

    return _possibleConstructorReturn(_this, new Inner());
  }

  return Outer;
}(Hello);

expect(new Outer().hello).toBe('hello');

"#
);

test!(
    syntax(),
    |_| tr(),
    public_instance_undefined,
    r#"
class Foo {
  bar;
}

"#,
    r#"
var Foo = function Foo() {
  

  _classCallCheck(this, Foo);
  _defineProperty(this, "bar", void 0);
};

"#
);

test!(
    syntax(),
    |_| tr(),
    private_derived_multiple_supers,
    r#"
class Foo extends Bar {
  #bar = "foo";

  constructor() {
    if (condition) {
      super();
    } else {
      super();
    }
  }
}

"#,
    r#"
var Foo =
/*#__PURE__*/
function (_Bar) {
  

  _inherits(Foo, _Bar);

  function Foo() {
    var _this;

    _classCallCheck(this, Foo);

    if (condition) {
      _this = _possibleConstructorReturn(this, _getPrototypeOf(Foo).call(this));

      _bar.set(_assertThisInitialized(_this), {
        writable: true,
        value: "foo"
      });
    } else {
      _this = _possibleConstructorReturn(this, _getPrototypeOf(Foo).call(this));

      _bar.set(_assertThisInitialized(_this), {
        writable: true,
        value: "foo"
      });
    }

    return _possibleConstructorReturn(_this);
  }

  return Foo;
}(Bar);

var _bar = new WeakMap();

"#
);

test_exec!(
    syntax(),
    |_| tr(),
    public_native_classes_exec,
    r#"
class Foo {
  static foo = "foo";
  bar = "bar";

  static test() {
    return Foo.foo;
  }

  test() {
    return this.bar;
  }
}

const f = new Foo();
expect("foo" in Foo).toBe(true)
expect("bar" in f).toBe(true)
expect(Foo.test()).toBe("foo")
expect(f.test()).toBe("bar")

"#
);

test!(
    syntax(),
    |_| tr(),
    public_regression_t2983,
    r#"
call(class {
  static test = true
});

export default class {
  static test = true
}

"#,
    r#"
call(function() {
    var _class = function _class() {
      _classCallCheck(this, _class);
    };
    _defineProperty(_class, 'test', true);
    return _class;
}());
var _class = function _class() {
  _classCallCheck(this, _class);
};
_defineProperty(_class, 'test', true);
export { _class as default }
"#
);

test!(
    syntax(),
    |_| tr(),
    public_static,
    r#"
class Foo {
  static bar = "foo";
}

"#,
    r#"
var Foo = function Foo() {
  

  _classCallCheck(this, Foo);
};

_defineProperty(Foo, "bar", "foo");

"#
);

test!(
    syntax(),
    |_| tr(),
    private_instance_undefined,
    r#"
class Foo {
  #bar;
}

"#,
    r#"
var Foo = function Foo() {
  

  _classCallCheck(this, Foo);

  _bar.set(this, {
    writable: true,
    value: void 0
  });
};

var _bar = new WeakMap();

"#
);

test_exec!(
    syntax(),
    |_| tr(),
    private_declaration_order_exec,
    r#"
class C {
  y = this.#x;
  #x;
}

expect(() => {
  new C();
}).toThrow();

"#
);

test!(
    syntax(),
    |_| tr(),
    public_update,
    r#"
class Foo {
  foo = 0;

  test(other) {
    this.foo++;
    ++this.foo;
    other.obj.foo++;
    ++other.obj.foo;
  }
}

"#,
    r#"
var Foo =
/*#__PURE__*/
function () {
  

  function Foo() {
    _classCallCheck(this, Foo);
    _defineProperty(this, "foo", 0);
  }

  _createClass(Foo, [{
    key: "test",
    value: function test(other) {
      this.foo++;
      ++this.foo;
      other.obj.foo++;
      ++other.obj.foo;
    }
  }]);
  return Foo;
}();

"#
);

test!(
    syntax(),
    |_| tr(),
    public_super_call,
    r#"
class A {
  foo() {
    return "bar";
  }
}

class B extends A {
  foo = super.foo();
}

"#,
    r#"
var A =
/*#__PURE__*/
function () {
  

  function A() {
    _classCallCheck(this, A);
  }

  _createClass(A, [{
    key: "foo",
    value: function foo() {
      return "bar";
    }
  }]);
  return A;
}();

var B =
/*#__PURE__*/
function (_A) {
  

  _inherits(B, _A);

  function B() {
    var _this;

    _classCallCheck(this, B);
    _this = _possibleConstructorReturn(this, _getPrototypeOf(B).apply(this, arguments));
    _defineProperty(_assertThisInitialized(_this), "foo", _get(_getPrototypeOf(B.prototype), "foo", _assertThisInitialized(_this)).call(_this));
    return _this;
  }

  return B;
}(A);

"#
);

test!(
    syntax(),
    |_| tr(),
    private_constructor_collision,
    r#"
var foo = "bar";

class Foo {
  #bar = foo;

  constructor() {
    var foo = "foo";
  }
}

"#,
    r#"
var foo = "bar";

var Foo = function Foo() {
  

  _classCallCheck(this, Foo);

  _bar.set(this, {
    writable: true,
    value: foo
  });
  var foo1 = "foo";

};

var _bar = new WeakMap();

"#
);

test!(
    syntax(),
    |_| tr(),
    public_constructor_collision,
    r#"
var foo = "bar";

class Foo {
  bar = foo;
  static bar = baz;

  constructor() {
    var foo = "foo";
    var baz = "baz";
  }
}

"#,
    r#"
var foo = "bar";

var Foo = function Foo() {
  

  _classCallCheck(this, Foo);
  _defineProperty(this, "bar", foo);
  var foo1 = "foo";
  var baz = "baz";
};

_defineProperty(Foo, "bar", baz);

"#
);

test!(
    syntax(),
    |_| tr(),
    public_computed,
    r#"
const foo = "foo";
const bar = () => {};
const four = 4;

class MyClass {
  static [one()] = "test";
  static [2 * 4 + 7] = "247";
  static [2 * four + 7] = "247";
  static [2 * four + seven] = "247";
  [null] = "null";
  [undefined] = "undefined";
  [void 0] = "void 0";
  get ["whatever"]() {}
  set ["whatever"](value) {}
  get [computed()]() {}
  set [computed()](value) {}
  ["test" + one]() {}
  static [10]() {}
  [/regex/] = "regex";
  [foo] = "foo";
  [bar] = "bar";
  [baz] = "baz";
  [`template`] = "template";
  [`template${expression}`] = "template-with-expression";
}

"#,
    r#"
var foo = 'foo';
var bar = ()=>{
};
var four = 4;
var _ref = one(), _ref1 = 2 * 4 + 7, _ref2 = 2 * four + 7, _ref3 = 2 * four + seven,
  _ref4 = null, _undefined = undefined, _ref5 = void 0, tmp = 'whatever',
  tmp1 = 'whatever', tmp2 = computed(), tmp3 = computed(), tmp4 = 'test' + one,
  tmp5 = 10, _ref6 = /regex/, _foo = foo, _bar = bar, _baz = baz,
  _ref7 = `template`, _ref8 = `template${expression}`;
var MyClass = function() {
    function MyClass() {
        _classCallCheck(this, MyClass);
        _defineProperty(this, _ref4, 'null');
        _defineProperty(this, _undefined, 'undefined');
        _defineProperty(this, _ref5, 'void 0');
        _defineProperty(this, _ref6, 'regex');
        _defineProperty(this, _foo, 'foo');
        _defineProperty(this, _bar, 'bar');
        _defineProperty(this, _baz, 'baz');
        _defineProperty(this, _ref7, 'template');
        _defineProperty(this, _ref8, 'template-with-expression');
    }
    _createClass(MyClass, [{
             key: tmp, get: function () {
                } 
        }, {
             key: tmp1, set: function (value) {
                } 
        }, {
             key: tmp2, get: function () {
                } 
        }, {
             key: tmp3, set: function (value) {
                } 
        }, {
             key: tmp4, value: function () {
                } 
        }], [{
             key: tmp5, value: function () {
                } 
        }]);
    return MyClass;
}();
_defineProperty(MyClass, _ref, 'test');
_defineProperty(MyClass, _ref1, '247');
_defineProperty(MyClass, _ref2, '247');
_defineProperty(MyClass, _ref3, '247');

"#
);

test!(
    syntax(),
    |_| tr(),
    public_assignment,
    r#"
class Foo {
  foo = 0;

  test(other) {
    this.foo++;
    this.foo += 1;
    this.foo = 2;
    other.obj.foo++;
    other.obj.foo += 1;
    other.obj.foo = 2;
  }
}

"#,
    r#"
var Foo =
/*#__PURE__*/
function () {
  

  function Foo() {
    _classCallCheck(this, Foo);
    _defineProperty(this, "foo", 0);
  }

  _createClass(Foo, [{
    key: "test",
    value: function test(other) {
      this.foo++;
      this.foo += 1;
      this.foo = 2;
      other.obj.foo++;
      other.obj.foo += 1;
      other.obj.foo = 2;
    }
  }]);
  return Foo;
}();

"#
);

test_exec!(
    syntax(),
    |_| tr(),
    public_static_exec,
    r#"
class Foo {
  static num = 0;
  static str = "foo";
}

expect(Foo.num).toBe(0);
expect(Foo.num = 1).toBe(1);
expect(Foo.str).toBe("foo");
expect(Foo.str = "bar").toBe("bar");

"#
);

test_exec!(
    // TOOOOO long to read
    // TODO(kdy1): unignore
    ignore,
    syntax(),
    |_| tr(),
    regression_7371_exec,
    r#"

class C {
}

class A extends C {
  field = 1;

  constructor() {
    super();

    class B extends C {
      constructor() {
        super();

        expect(this.field).toBeUndefined();
      }
    }

    expect(this.field).toBe(1)

    new B();
  }
}

new A();

class Obj {
  constructor() {
    return {};
  }
}

// ensure superClass is still transformed
class SuperClass extends Obj {
  field = 1;

  constructor() {
    class B extends (super(), Obj) {
      constructor() {
        super();

        expect(this.field).toBeUndefined()
      }
    }

    expect(this.field).toBe(1)

    new B();
  }
}

new SuperClass();

// ensure ComputedKey Method is still transformed
class ComputedMethod extends Obj {
  field = 1;

  constructor() {
    class B extends Obj {
      constructor() {
        super();

        expect(this.field).toBeUndefined()
      }

      [super()]() { }
    }

    expect(this.field).toBe(1)

    new B();
  }
}

new ComputedMethod();


// ensure ComputedKey Field is still transformed
class ComputedField extends Obj {
  field = 1;

  constructor() {
    class B extends Obj {
      constructor() {
        super();

        expect(this.field).toBeUndefined()
      }

      [super()] = 1;
    }

    expect(this.field).toBe(1)

    new B();
  }
}

new ComputedField();

"#
);

test_exec!(
    syntax(),
    |_| tr(),
    private_static_inherited_exec,
    r#"
class Base {
  static #foo = 1;

  static getThis() {
    return this.#foo;
  }

  static updateThis(val) {
    return (this.#foo = val);
  }

  static getClass() {
    return Base.#foo;
  }

  static updateClass(val) {
    return (Base.#foo = val);
  }
}

class Sub1 extends Base {
  static #foo = 2;

  static update(val) {
    return (this.#foo = val);
  }
}

class Sub2 extends Base {}

expect(Base.getThis()).toBe(1);
expect(Base.getClass()).toBe(1);
expect(() => Sub1.getThis()).toThrow();
expect(Sub1.getClass()).toBe(1);
expect(() => Sub2.getThis()).toThrow();
expect(Sub2.getClass()).toBe(1);

expect(Sub1.update(3)).toBe(3);
expect(Base.getThis()).toBe(1);
expect(Base.getClass()).toBe(1);
expect(() => Sub1.getThis()).toThrow();
expect(Sub1.getClass()).toBe(1);
expect(() => Sub2.getThis()).toThrow();
expect(Sub2.getClass()).toBe(1);

expect(Base.updateThis(4)).toBe(4);
expect(Base.getThis()).toBe(4);
expect(Base.getClass()).toBe(4);
expect(() => Sub1.getThis()).toThrow();
expect(Sub1.getClass()).toBe(4);
expect(() => Sub2.getThis()).toThrow();
expect(Sub2.getClass()).toBe(4);

expect(Base.updateClass(5)).toBe(5);
expect(Base.getThis()).toBe(5);
expect(Base.getClass()).toBe(5);
expect(() => Sub1.getThis()).toThrow();
expect(Sub1.getClass()).toBe(5);
expect(() => Sub2.getThis()).toThrow();
expect(Sub2.getClass()).toBe(5);

expect(() => Sub2.updateThis(6)).toThrow();
expect(Sub2.updateClass(7)).toBe(7);
expect(Base.getThis()).toBe(7);
expect(Base.getClass()).toBe(7);
expect(() => Sub1.getThis()).toThrow();
expect(Sub1.getClass()).toBe(7);
expect(() => Sub2.getThis()).toThrow();
expect(Sub2.getClass()).toBe(7);

"#
);

test_exec!(
    syntax(),
    |_| tr(),
    nested_class_super_property_in_key_exec,
    r#"

class Hello {
  toString() {
    return 'hello';
  }
}

class Outer extends Hello {
  constructor() {
    super();
    class Inner {
      [super.toString()] = 'hello';
    }

    return new Inner();
  }
}

expect(new Outer().hello).toBe('hello');

"#
);

test!(
    syntax(),
    |_| tr(),
    private_super_statement,
    r#"
class Foo extends Bar {
  #bar = "foo";

  constructor() {
    super();
  }
}

"#,
    r#"
var Foo =
/*#__PURE__*/
function (_Bar) {
  

  _inherits(Foo, _Bar);

  function Foo() {
    var _this;

    _classCallCheck(this, Foo);
    _this = _possibleConstructorReturn(this, _getPrototypeOf(Foo).call(this));

    _bar.set(_assertThisInitialized(_this), {
      writable: true,
      value: "foo"
    });

    return _this;
  }

  return Foo;
}(Bar);

var _bar = new WeakMap();

"#
);

test!(
    syntax(),
    |_| tr(),
    private_private_in_derived,
    r#"
class Outer {
  #outer;

  constructor() {
    class Test extends this.#outer {
    }
  }
}

"#,
    r#"
var Outer = function Outer() {
  var _this = this;


  _classCallCheck(this, Outer);

  _outer.set(this, {
    writable: true,
    value: void 0
  });

  var Test =
  /*#__PURE__*/
  function (_super) {
    _inherits(Test, _super);

    function Test() {
      _classCallCheck(this, Test);
      return _possibleConstructorReturn(this, _getPrototypeOf(Test).apply(this, arguments));
    }

    return Test;
  }(_classPrivateFieldGet(_this, _outer));
};

var _outer = new WeakMap();

"#
);

test!(
    syntax(),
    |_| tr(),
    private_update,
    r#"
class Foo {
  #foo = 0;

  test(other) {
    this.#foo++;
    ++this.#foo;
    other.obj.#foo++;
    ++other.obj.#foo;
  }
}

"#,
    r#"
var Foo =
/*#__PURE__*/
function () {
  

  function Foo() {
    _classCallCheck(this, Foo);

    _foo.set(this, {
      writable: true,
      value: 0
    });
  }

  _createClass(Foo, [{
    key: "test",
    value: function test(other) {
      var old, _obj, old1, _obj1;

      _classPrivateFieldSet(this, _foo, (old = +_classPrivateFieldGet(this, _foo)) + 1), old;
      _classPrivateFieldSet(this, _foo, +_classPrivateFieldGet(this, _foo) + 1);
      _classPrivateFieldSet(_obj = other.obj, _foo, (old1 = +_classPrivateFieldGet(_obj, _foo)) + 1), old1;
      _classPrivateFieldSet(_obj1 = other.obj, _foo, +_classPrivateFieldGet(_obj1, _foo) + 1);
    }
  }]);
  return Foo;
}();

var _foo = new WeakMap();

"#
);

test!(
    syntax(),
    |_| tr(),
    public_super_expression,
    r#"
class Foo extends Bar {
  bar = "foo";

  constructor() {
    foo(super());
  }
}

"#,
    r#"
var Foo = function (_Bar) {
  

  _inherits(Foo, _Bar);

  function Foo() {
    var _this;

    _classCallCheck(this, Foo);
    var _temp;
    foo((_temp = _this = _possibleConstructorReturn(this, _getPrototypeOf(Foo).call(this)), _defineProperty(_assertThisInitialized(_this), "bar", "foo"), _temp));
    return _possibleConstructorReturn(_this);
  }

  return Foo;
}(Bar);

"#
);

test_exec!(
    syntax(),
    |_| tr(),
    public_computed_initialization_order_exec,
    r#"
const actualOrder = [];

const track = i => {
  actualOrder.push(i);
  return i;
};

class MyClass {
  static [track(1)] = track(10);
  [track(2)] = track(13);
  get [track(3)]() {
    return "foo";
  }
  set [track(4)](value) {
    this.bar = value;
  }
  [track(5)] = track(14);
  static [track(6)] = track(11);
  static [track(7)] = track(12);
  [track(8)]() {}
  [track(9)] = track(15);
}

const inst = new MyClass();

const expectedOrder = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
expect(actualOrder).toEqual(expectedOrder);

expect(MyClass[1]).toBe(10);
expect(inst[2]).toBe(13);
expect(inst[3]).toBe("foo");
inst[4] = "baz";
expect(inst.bar).toBe("baz");
expect(inst[5]).toBe(14);
expect(MyClass[6]).toBe(11);
expect(MyClass[7]).toBe(12);
expect(typeof inst[8]).toBe("function");
expect(inst[9]).toBe(15);

"#
);

test_exec!(
    syntax(),
    |_| tr(),
    nested_class_super_call_in_key_exec,
    r#"

class Hello {
  constructor() {
    return {
      toString() {
        return 'hello';
      },
    };
  }
}

class Outer extends Hello {
  constructor() {
    class Inner {
      [super()] = "hello";
    }

    return new Inner();
  }
}

expect(new Outer().hello).toBe('hello');

"#
);

test_exec!(
    syntax(),
    |_| tr(),
    private_update_exec,
    r#"
class Foo {
  #foo = 0;

  test(other) {
    return [
      this.#foo++,
      this.#foo,
      ++this.#foo,
      this.#foo,
      other.obj.#foo++,
      other.obj.#foo,
      ++other.obj.#foo,
      other.obj.#foo,
    ];
  }
}

const f = new Foo;
const results = f.test({ obj: f });
expect(results[0]).toBe(0);
expect(results[1]).toBe(1);
expect(results[2]).toBe(2);
expect(results[3]).toBe(2);
expect(results[4]).toBe(2);
expect(results[5]).toBe(3);
expect(results[6]).toBe(4);
expect(results[7]).toBe(4);

"#
);

test!(
    syntax(),
    |_| tr(),
    public_extracted_this,
    r#"
var foo = "bar";

class Foo {
  bar = this;
  baz = foo;

  constructor(foo) {
  }
}

"#,
    r#"
var foo = "bar";

var Foo = function Foo(foo1) {
  

  _classCallCheck(this, Foo);
  _defineProperty(this, "bar", this);
  _defineProperty(this, "baz", foo);
};

"#
);

test!(
    syntax(),
    |_| tr(),
    private_derived,
    r#"
class Foo {
  #prop = "foo";
}

class Bar extends Foo {
  #prop = "bar";
}

"#,
    r#"
var Foo = function Foo() {
  

  _classCallCheck(this, Foo);

  _prop1.set(this, {
    writable: true,
    value: "foo"
  });
};

var _prop1 = new WeakMap();

var Bar =
/*#__PURE__*/
function (_Foo) {
  

  _inherits(Bar, _Foo);

  function Bar() {
    var _this;

    _classCallCheck(this, Bar);
    _this = _possibleConstructorReturn(this, _getPrototypeOf(Bar).apply(this, arguments));

    _prop2.set(_assertThisInitialized(_this), {
      writable: true,
      value: "bar"
    });

    return _this;
  }

  return Bar;
}(Foo);

var _prop2 = new WeakMap();

"#
);

test!(
    syntax(),
    |_| tr(),
    private_super_call,
    r#"
class A {
  foo() {
    return "bar";
  }
}

class B extends A {
  #foo = super.foo();
}

"#,
    r#"
var A = function () {
  

  function A() {
    _classCallCheck(this, A);
  }

  _createClass(A, [{
    key: "foo",
    value: function foo() {
      return "bar";
    }
  }]);
  return A;
}();

var B =
/*#__PURE__*/
function (_A) {
  

  _inherits(B, _A);

  function B() {
    var _this;

    _classCallCheck(this, B);
    _this = _possibleConstructorReturn(this, _getPrototypeOf(B).apply(this, arguments));

    _foo.set(_assertThisInitialized(_this), {
      writable: true,
      value: _get(_getPrototypeOf(B.prototype), "foo", _assertThisInitialized(_this))()
    });

    return _this;
  }

  return B;
}(A);

var _foo = new WeakMap();

"#
);

test!(
    syntax(),
    |_| tr(),
    private_reference_in_other_property,
    r#"
class Foo {
  one = this.#private;
  #two = this.#private;
  #private = 0;
  three = this.#private;
  #four = this.#private;
}

"#,
    r#"
var Foo = function Foo() {
  

  _classCallCheck(this, Foo);
  _defineProperty(this, "one", _classPrivateFieldGet(this, _private));

  _two.set(this, {
    writable: true,
    value: _classPrivateFieldGet(this, _private)
  });

  _private.set(this, {
    writable: true,
    value: 0
  });

  _defineProperty(this, "three", _classPrivateFieldGet(this, _private));

  _four.set(this, {
    writable: true,
    value: _classPrivateFieldGet(this, _private)
  });
};

var _two = new WeakMap();

var _private = new WeakMap();

var _four = new WeakMap();

"#
);

test!(
    syntax(),
    |_| tr(),
    nested_class_super_property_in_key,
    r#"

class Hello {
  toString() {
    return 'hello';
  }
}

class Outer extends Hello {
  constructor() {
    super();
    class Inner {
      [super.toString()] = 'hello';
    }

    return new Inner();
  }
}

expect(new Outer().hello).toBe('hello');

"#,
    r#"


var Hello = function () {
  function Hello() {
    _classCallCheck(this, Hello);
  }

  _createClass(Hello, [{
    key: "toString",
    value: function toString() {
      return 'hello';
    }
  }]);
  return Hello;
}();

var Outer = function (_Hello) {
  _inherits(Outer, _Hello);

  function Outer() {
    _classCallCheck(this, Outer);
    var _this = _possibleConstructorReturn(this, _getPrototypeOf(Outer).call(this));

    var _ref = _get(_getPrototypeOf(Outer.prototype), 'toString', _assertThisInitialized(_this)).call(_this);

    var Inner = function Inner() {
      _classCallCheck(this, Inner);
      _defineProperty(this, _ref, 'hello');
    };

    return _possibleConstructorReturn(_this, new Inner());
  }

  return Outer;
}(Hello);

expect(new Outer().hello).toBe('hello');

"#
);

test_exec!(
    syntax(),
    |_| tr(),
    private_reevaluated_exec,
    r#"
function classFactory() {
  return class Foo {
    #foo = "foo";
    static #bar = "bar";

    instance() {
      return this.#foo;
    }

    static() {
      return Foo.#bar;
    }

    static instance(inst) {
      return inst.#foo;
    }

    static static() {
      return Foo.#bar;
    }
  };
}

const Foo1 = classFactory();
const Foo2 = classFactory();

const f1 = new Foo1();
const f2 = new Foo2();

expect(f1.instance()).toBe("foo");
expect(f1.static()).toBe("bar");
expect(f2.instance()).toBe("foo");
expect(f2.static()).toBe("bar");

expect(Foo1.instance(f1)).toBe("foo");
expect(Foo1.static()).toBe("bar");
expect(Foo2.instance(f2)).toBe("foo");
expect(Foo2.static()).toBe("bar");

expect(() => {
  f1.instance.call(f2), undefined;
}).toThrow();
expect(() => {
  f2.instance.call(f1), undefined;
}).toThrow();
expect(() => {
  Foo1.instance(f2), undefined;
}).toThrow();
expect(() => {
  Foo2.instance(f1), undefined;
}).toThrow();

"#
);

test!(
    syntax(),
    |_| tr(),
    public_numeric,
    r#"
class Foo {
  0 = "foo";
  1 = "bar";
}

"#,
    r#"
var Foo = function Foo() {
  

  _classCallCheck(this, Foo);
  _defineProperty(this, 0, "foo");
  _defineProperty(this, 1, "bar");
};

"#
);

test!(
    syntax(),
    |_| tr(),
    private_assignment,
    r#"
class Foo {
  #foo = 0;

  test(other) {
    this.#foo += 1;
    this.#foo = 2;
    other.obj.#foo += 1;
    other.obj.#foo = 2;
  }
}

"#,
    r#"
var Foo =
/*#__PURE__*/
function () {
  

  function Foo() {
    _classCallCheck(this, Foo);

    _foo.set(this, {
      writable: true,
      value: 0
    });
  }

  _createClass(Foo, [{
    key: "test",
    value: function test(other) {
      var _obj;

      _classPrivateFieldSet(this, _foo, _classPrivateFieldGet(this, _foo) + 1);
      _classPrivateFieldSet(this, _foo, 2);
      _classPrivateFieldSet(_obj = other.obj, _foo, _classPrivateFieldGet(_obj, _foo) + 1);
      _classPrivateFieldSet(other.obj, _foo, 2);
    }
  }]);
  return Foo;
}();

var _foo = new WeakMap();

"#
);

test_exec!(
    syntax(),
    |_| tr(),
    private_constructor_collision_exec,
    r#"
var foo = "bar";

class Foo {
  #bar = foo;

  constructor() {
    var foo = "foo";
  }

  test() {
    return this.#bar;
  }
}

const f = new Foo;
expect(f.test()).toBe(foo);
expect("bar" in f).toBe(false);

"#
);

test!(
    syntax(),
    |_| tr(),
    public_static_export,
    r#"
export class MyClass {
  static property = value;
}

export default class MyClass2 {
  static property = value;
}

"#,
    r#"
export var MyClass = function MyClass() {
  _classCallCheck(this, MyClass);
};
_defineProperty(MyClass, "property", value);

var MyClass2 = function MyClass2() {
  _classCallCheck(this, MyClass2);
};

_defineProperty(MyClass2, "property", value);
export { MyClass2 as default };

"#
);

test!(
    syntax(),
    |_| tr(),
    private_multiple,
    r#"
class Foo {
  #x = 0;
  #y = this.#x;
}

"#,
    r#"
var Foo = function Foo() {
  

  _classCallCheck(this, Foo);

  _x.set(this, {
    writable: true,
    value: 0
  });

  _y.set(this, {
    writable: true,
    value: _classPrivateFieldGet(this, _x)
  });
};

var _x = new WeakMap();

var _y = new WeakMap();

"#
);

test!(
    syntax(),
    |_| tr(),
    public_derived,
    r#"
class Foo extends Bar {
  bar = "foo";
}

"#,
    r#"
var Foo =
/*#__PURE__*/
function (_Bar) {
  

  _inherits(Foo, _Bar);

  function Foo() {
    var _this;

    _classCallCheck(this, Foo);
    _this = _possibleConstructorReturn(this, _getPrototypeOf(Foo).apply(this, arguments))
    _defineProperty(_assertThisInitialized(_this), "bar", "foo");
    return _this;
  }

  return Foo;
}(Bar);

"#
);

test_exec!(
    syntax(),
    |_| tr(),
    public_static_undefined_exec,
    r#"
class Foo {
  static num;
}

expect("num" in Foo).toBe(true);
expect(Foo.num).toBeUndefined();

"#
);

test!(
    syntax(),
    |_| tr(),
    public_instance,
    r#"
class Foo {
  bar = "foo";
}

"#,
    r#"
var Foo = function Foo() {
  

  _classCallCheck(this, Foo);
  _defineProperty(this, "bar", "foo");
};

"#
);

test_exec!(
    syntax(),
    |_| tr(),
    static_property_tdz_edgest_case_exec,
    r#"
expect(() => {
  class A {
    static [{ x: A || 0 }.x];
  }
}).toThrow();

"#
);

test!(
    syntax(),
    |_| tr(),
    public_non_block_arrow_func,
    r#"
export default param =>
  class App {
    static props = {
      prop1: 'prop1',
      prop2: 'prop2'
    }

    getParam() {
      return param;
    }
  }

"#,
    r#"
export default (param)=>{
  var App = function() {
    function App() {
      _classCallCheck(this, App);
    }
    _createClass(App, [{
      key: 'getParam',
      value: function getParam() {
        return param;
      } 
    }]);
    return App;
  }();
  _defineProperty(App, 'props', {
    prop1: 'prop1', prop2: 'prop2' 
  });
  return App;
};
"#
);

test!(
    syntax(),
    |_| tr(),
    public_static_undefined,
    r#"
class Foo {
  static bar;
}

"#,
    r#"
var Foo = function Foo() {
  

  _classCallCheck(this, Foo);
};

_defineProperty(Foo, "bar", void 0);

"#
);

test_exec!(
    syntax(),
    |_| tr(),
    public_static_infer_name_exec,
    r#"
var Foo = class {
  static num = 0;
}

expect(Foo.num).toBe(0);
expect(Foo.num = 1).toBe(1);
expect(Foo.name).toBe("Foo");

"#
);

test_exec!(
    syntax(),
    |_| tr(),
    static_property_tdz_general_exec,
    r#"
expect(() => {
  class C {
    static [C + 3] = 3;
  }
}).toThrow();

"#
);

test!(
    syntax(),
    |_| tr(),
    private_call,
    r#"
class Foo {
  #foo = function() {
    return this;
  }

  test(other) {
    this.#foo();
    other.obj.#foo();
  }
}

"#,
    r#"
var Foo = function () {
  

  function Foo() {
    var _this = this;
    _classCallCheck(this, Foo);

    _foo.set(this, {
      writable: true,
      value: function () {
        return _this;
      }
    });
  }

  _createClass(Foo, [{
    key: "test",
    value: function test(other) {
      var _obj;

      _classPrivateFieldGet(this, _foo).call(this);
      _classPrivateFieldGet(_obj = other.obj, _foo).call(_obj);
    }
  }]);
  return Foo;
}();

var _foo = new WeakMap();

"#
);

test_exec!(
    syntax(),
    |_| tr(),
    private_derived_exec,
    r#"
class Foo {
  #prop = "foo";

  foo() {
    return this.#prop;
  }
}

class Bar extends Foo {
  #prop = "bar";

  bar() {
    return this.#prop;
  }
}

const f = new Foo;
expect(f.foo()).toBe("foo");

const b = new Bar;
expect(b.foo()).toBe("foo");
expect(b.bar()).toBe("bar");

"#
);

test!(
    syntax(),
    |_| tr(),
    private_extracted_this,
    r#"
var foo = "bar";

class Foo {
  #bar = this;
  #baz = foo;

  constructor(foo) {
  }
}

"#,
    r#"
var foo = "bar";

var Foo = function Foo(foo1) {
  

  _classCallCheck(this, Foo);

  _bar.set(this, {
    writable: true,
    value: this
  });

  _baz.set(this, {
    writable: true,
    value: foo
  });
};

var _bar = new WeakMap();

var _baz = new WeakMap();

"#
);

test_exec!(
    syntax(),
    |_| tr(),
    private_canonical_exec,
    r#"
class Point {
    #x;
    #y;

    constructor(x = 0, y = 0) {
        this.#x = +x;
        this.#y = +y;
    }

    get x() { return this.#x }
    set x(value) { this.#x = +value }

    get y() { return this.#y }
    set y(value) { this.#y = +value }

    equals(p) { return this.#x === p.#x && this.#y === p.#y }

    toString() { return `Point<${ this.#x },${ this.#y }>` }

}

const p1 = new Point(1, 2);
const p2 = new Point(2, 3);
const p3 = new Point(1, 2);

expect(p1.x).toBe(1);
expect(p1.y).toBe(2);
expect(p2.x).toBe(2);
expect(p2.y).toBe(3);
expect(p3.x).toBe(1);
expect(p3.y).toBe(2);

expect(p1.equals(p1)).toBe(true)
expect(p1.equals(p2)).toBe(false)
expect(p1.equals(p3)).toBe(true)
expect(p2.equals(p1)).toBe(false)
expect(p2.equals(p2)).toBe(true)
expect(p2.equals(p3)).toBe(false)
expect(p3.equals(p1)).toBe(true)
expect(p3.equals(p2)).toBe(false)
expect(p3.equals(p3)).toBe(true)

expect(p1.toString()).toBe("Point<1,2>")
expect(p2.toString()).toBe("Point<2,3>")
expect(p3.toString()).toBe("Point<1,2>")

p1.x += 1;
p1.y = 3;
p2.x -= 1;
p2.y = 3;
p3.x = 0;
p3.y = 0;

expect(p1.x).toBe(2);
expect(p1.y).toBe(3);
expect(p2.x).toBe(1);
expect(p2.y).toBe(3);
expect(p3.x).toBe(0);
expect(p3.y).toBe(0);

expect(p1.equals(p1)).toBe(true)
expect(p1.equals(p2)).toBe(false)
expect(p1.equals(p3)).toBe(false)
expect(p2.equals(p1)).toBe(false)
expect(p2.equals(p2)).toBe(true)
expect(p2.equals(p3)).toBe(false)
expect(p3.equals(p1)).toBe(false)
expect(p3.equals(p2)).toBe(false)
expect(p3.equals(p3)).toBe(true)

expect(p1.toString()).toBe("Point<2,3>")
expect(p2.toString()).toBe("Point<1,3>")
expect(p3.toString()).toBe("Point<0,0>")

"#
);

test_exec!(
    syntax(),
    |_| tr(),
    private_static_undefined_exec,
    r#"
class Foo {
  static #bar;

  static test() {
    return Foo.#bar;
  }

  test() {
    return Foo.#bar;
  }
}

expect("bar" in Foo).toBe(false);
expect(Foo.test()).toBe(undefined);
expect(Foo.test()).toBe(undefined);

"#
);

test_exec!(
    syntax(),
    |_| tr(),
    public_update_exec,
    r#"
class Foo {
  foo = 0;

  test(other) {
    return [
      this.foo++,
      this.foo,
      ++this.foo,
      this.foo,
      other.obj.foo++,
      other.obj.foo,
      ++other.obj.foo,
      other.obj.foo,
    ];
  }
}

const f = new Foo;
const results = f.test({ obj: f });
expect(results[0]).toBe(0);
expect(results[1]).toBe(1);
expect(results[2]).toBe(2);
expect(results[3]).toBe(2);
expect(results[4]).toBe(2);
expect(results[5]).toBe(3);
expect(results[6]).toBe(4);
expect(results[7]).toBe(4);

"#
);

test!(
    syntax(),
    |_| tr(),
    private_static_call,
    r#"
class Foo {
  static #foo = function(x) {
    return x;
  }

  test(x) {
    return Foo.#foo(x);
  }
}


"#,
    r#"
var Foo =
/*#__PURE__*/
function () {
  

  function Foo() {
    _classCallCheck(this, Foo);
  }

  _createClass(Foo, [{
    key: "test",
    value: function test(x) {
      return _classStaticPrivateFieldSpecGet(Foo, Foo, _foo).call(Foo, x);
    }
  }]);
  return Foo;
}();

var _foo = {
  writable: true,
  value: function (x) {
    return x;
  }
};

"#
);

test!(
    syntax(),
    |_| tr(),
    private_super_expression,
    r#"
class Foo extends Bar {
  #bar = "foo";

  constructor() {
    foo(super());
  }
}

"#,
    r#"
var Foo =
/*#__PURE__*/
function (_Bar) {
  

  _inherits(Foo, _Bar);

  function Foo() {
    var _this;

    _classCallCheck(this, Foo);
    var _temp;
    foo((_temp = _this = _possibleConstructorReturn(this, _getPrototypeOf(Foo).call(this)),
     _bar.set(_assertThisInitialized(_this), {
      writable: true,
      value: "foo"
    }), _temp));
    return _possibleConstructorReturn(_this);
  }

  return Foo;
}(Bar);

var _bar = new WeakMap();

"#
);

test_exec!(
    syntax(),
    |_| tr(),
    private_native_classes_exec,
    r#"
class Foo {
  static #foo = "foo";
  #bar = "bar";

  static test() {
    return Foo.#foo;
  }

  test() {
    return this.#bar;
  }
}

const f = new Foo();
expect("foo" in Foo).toBe(false)
expect("bar" in f).toBe(false)
expect(Foo.test()).toBe("foo")
expect(f.test()).toBe("bar")

"#
);

test_exec!(
    syntax(),
    |_| tr(),
    private_multiple_exec,
    r#"
class Foo {
  #x = 0;
  #y = this.#x + 1;

  test() {
    return this.#y;
  }
}

const f = new Foo();
expect(f.test()).toBe(1);

"#
);

test!(
    syntax(),
    |_| tr(),
    custom_instance_update,
    "
class Foo {
    #x = 0;

    test() {
        this.#x++;
        ++this.#x;
    }
}
",
    "
var Foo = function () {
  function Foo() {
    _classCallCheck(this, Foo);

    _x.set(this, {
      writable: true,
      value: 0
    });
  }

  _createClass(Foo, [{
    key: 'test',
    value: function test() {
      var old;

      _classPrivateFieldSet(this, _x, (old = +_classPrivateFieldGet(this, _x)) + 1), old;

      _classPrivateFieldSet(this, _x, +_classPrivateFieldGet(this, _x) + 1);
    }
  }]);

  return Foo;
}();

var _x = new WeakMap();
"
);

test!(
    syntax(),
    |_| tr(),
    custom_static_update,
    "
class Foo {
    static #x = 0;

    test() {
        Foo.#x++;
        ++Foo.#x;
    }
}
",
    "
var Foo = function () {
  function Foo() {
    _classCallCheck(this, Foo);
  }

  _createClass(Foo, [{
    key: 'test',
    value: function test() {
      var old;

      _classStaticPrivateFieldSpecSet(Foo, Foo, _x, (old = +_classStaticPrivateFieldSpecGet(Foo, \
     Foo, _x)) + 1), old;

      _classStaticPrivateFieldSpecSet(Foo, Foo, _x, +_classStaticPrivateFieldSpecGet(Foo, Foo, _x) \
     + 1);
    }
  }]);

  return Foo;
}();

var _x = {
  writable: true,
  value: 0
};"
);

test!(
    syntax(),
    |_| chain!(resolver(), class_properties()),
    issue_308,
    "function bar(props) {}
class Foo {
  constructor() {
    super();
    bar();
  }
  onBar = () => {
    bar();
  };
}",
    "function bar(props) {
}
class Foo{
    constructor(){
        super();
        _defineProperty(this, 'onBar', ()=>{
            bar();
        });
        bar();
    }
}
"
);

test!(
    syntax(),
    |_| chain!(resolver(), class_properties(), Classes),
    issue_342,
    "class Foo {
  constructor(bar) {
    this._bar = bar;
  }

  qux = {
    frob: (bar) => {},
  };
}",
    "let Foo = function Foo(bar) {
    _classCallCheck(this, Foo);
    _defineProperty(this, 'qux', {
        frob: (bar)=>{
        }
    });
    this._bar = bar;
};"
);

test!(
    syntax(),
    |_| chain!(resolver(), class_properties(), block_scoping()),
    issue_443,
    "
const MODE = 1;

class foo {
  static MODE = MODE;

  constructor() {
    this.mode = MODE;
  }
}
",
    "var MODE = 1;
class foo{
    constructor(){
        this.mode = MODE;
    }
}
_defineProperty(foo, 'MODE', MODE);"
);
