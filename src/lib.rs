//! This library provides a Rust API for the <a href="https://developer.apple.com/documentation/javascriptcore">JavaScriptCore</a> engine.
//!
//! # Example
//!
//! The JavaScriptCore engine lets you evaluate JavaScript scripts from your
//! own application. You first need to create a `JSContext` object and then
//! call its `evaluate_script` method.
//!
//! ```rust
//! use rusty_jsc::JSContext;
//!
//! let mut context = JSContext::default();
//! match context.evaluate_script("'hello, world'", 1) {
//!     Some(value) => {
//!         println!("{}", value.to_string(&context));
//!     }
//!     None => {
//!         println!(
//!             "Uncaught: {}",
//!             context.get_exception().unwrap().to_string(&context)
//!         )
//!     }
//! }
//! ```

mod internal;

pub use crate::internal::JSString;
use anyhow::Result;
use rusty_jsc_sys::*;

/// A JavaScript value.
#[derive(Debug, Clone)]
pub struct JSValue {
    inner: JSValueRef,
}

impl Default for JSValue {
    fn default() -> Self {
        Self {
            inner: std::ptr::null_mut(),
        }
    }
}

impl From<JSObject> for JSValue {
    fn from(js_object: JSObject) -> Self {
        // The two objects are very simple and will not be differents in any
        // cases.
        unsafe { std::mem::transmute(js_object) }
    }
}

impl From<JSValueRef> for JSValue {
    /// Wraps a `JSValue` from a `JSValueRef`.
    fn from(inner: JSValueRef) -> Self {
        Self { inner }
    }
}

impl Drop for JSValue {
    fn drop(&mut self) {
        // TODO
    }
}

impl JSValue {
    pub fn get_ref(&self) -> JSValueRef {
        self.inner
    }

    /// Creates an `undefined` value.
    pub fn undefined(context: &JSContext) -> JSValue {
        JSValue::from(unsafe { JSValueMakeUndefined(context.inner) })
    }

    /// Creates a `null` value.
    pub fn null(context: &JSContext) -> JSValue {
        JSValue::from(unsafe { JSValueMakeNull(context.inner) })
    }

    /// Creates a `boolean` value.
    pub fn boolean(context: &JSContext, value: bool) -> JSValue {
        JSValue::from(unsafe { JSValueMakeBoolean(context.inner, value) })
    }

    /// Creates a `number` value.
    pub fn number(context: &JSContext, value: f64) -> JSValue {
        JSValue::from(unsafe { JSValueMakeNumber(context.inner, value) })
    }

    /// Creates a `string` value.
    pub fn string(context: &JSContext, value: impl ToString) -> Result<JSValue> {
        let value = JSString::from_utf8(value.to_string())?;
        Ok(JSValue::from(unsafe {
            JSValueMakeString(context.inner, value.inner)
        }))
    }

    /// Creates a function callback
    pub fn callback(context: &JSContext, callback: JSObjectCallAsFunctionCallback) -> JSValue {
        let name = JSString::from_utf8("".to_string()).unwrap();
        let func: JSValueRef =
            unsafe { JSObjectMakeFunctionWithCallback(context.inner, name.inner, callback) };
        JSValue::from(func)
    }

    /// Checks if this value is `undefined`.
    pub fn is_undefined(&self, context: &JSContext) -> bool {
        unsafe { JSValueIsUndefined(context.inner, self.inner) }
    }

    /// Checks if this value is `null`.
    pub fn is_null(&self, context: &JSContext) -> bool {
        unsafe { JSValueIsNull(context.inner, self.inner) }
    }

    /// Checks if this value is `boolean`.
    pub fn is_boolean(&self, context: &JSContext) -> bool {
        unsafe { JSValueIsBoolean(context.inner, self.inner) }
    }

    /// Checks if this value is `number`.
    pub fn is_number(&self, context: &JSContext) -> bool {
        unsafe { JSValueIsNumber(context.inner, self.inner) }
    }

    /// Checks if this value is `string`.
    pub fn is_string(&self, context: &JSContext) -> bool {
        unsafe { JSValueIsString(context.inner, self.inner) }
    }

    /// Checks if this value is `date`.
    pub fn is_date(&self, context: &JSContext) -> bool {
        unsafe { JSValueIsDate(context.inner, self.inner) }
    }

    /// Checks if this value is `date`.
    pub fn is_symbol(&self, context: &JSContext) -> bool {
        unsafe { JSValueIsSymbol(context.inner, self.inner) }
    }

    /// Formats this value as a `String`.
    pub fn to_string(&self, context: &JSContext) -> String {
        let mut exception: JSValueRef = std::ptr::null_mut();
        let s = unsafe { JSValueToStringCopy(context.inner, self.inner, &mut exception) };
        let s = JSString::from(s);
        s.to_string()
    }

    /// Convert value into object
    pub fn to_object(self, context: &JSContext) -> JSObject {
        unsafe { JSValueToObject(context.inner, self.inner, std::ptr::null_mut()).into() }
    }

    /// Get Rust boolean value from JSValue.
    pub fn to_boolean(&self, context: &JSContext) -> bool {
        unsafe { JSValueToBoolean(context.inner, self.inner) }
    }
}

/// A JavaScript object.
#[derive(Debug, Clone)]
pub struct JSObject {
    inner: JSObjectRef,
}

impl From<JSObjectRef> for JSObject {
    /// Wraps a `JSObject` from a `JSObjectRef`.
    fn from(inner: JSObjectRef) -> Self {
        Self { inner }
    }
}

impl Drop for JSObject {
    fn drop(&mut self) {
        // TODO
    }
}

impl JSObject {
    /// Sets the property of an object.
    pub fn set_property(
        &mut self,
        context: &JSContext,
        property_name: impl ToString,
        value: JSValue,
    ) {
        let property_name = JSString::from_utf8(property_name.to_string()).unwrap();
        let attributes = 0; // TODO
        let mut exception: JSValueRef = std::ptr::null_mut();
        unsafe {
            JSObjectSetProperty(
                context.inner,
                self.inner,
                property_name.inner,
                value.inner,
                attributes,
                &mut exception,
            )
        }
    }

    /// Sets the property of an object.
    pub fn delete_property(&mut self, context: &JSContext, property_name: impl ToString) {
        let property_name = JSString::from_utf8(property_name.to_string()).unwrap();
        let mut exception: JSValueRef = std::ptr::null_mut();
        unsafe {
            JSObjectDeleteProperty(
                context.inner,
                self.inner,
                property_name.inner,
                &mut exception,
            );
        }
    }

    pub fn get_property(
        &self,
        context: &JSContext,
        property_name: impl ToString,
    ) -> Option<JSValue> {
        let property_name = JSString::from_utf8(property_name.to_string()).unwrap();
        let exception: JSValueRef = std::ptr::null_mut();
        let js_value = unsafe {
            JSObjectGetProperty(
                context.inner,
                self.inner,
                property_name.inner,
                exception as *mut _,
            )
        };
        if !js_value.is_null() {
            Some(JSValue::from(js_value))
        } else {
            None
        }
    }

    pub fn class(
        context: &mut JSContext,
        class_name: impl ToString,
        constructor: JSObjectCallAsConstructorCallback,
    ) -> JSObject {
        let mut class_definition = unsafe { kJSClassDefinitionEmpty };
        class_definition.className = class_name.to_string().as_bytes().as_ptr() as _;
        class_definition.callAsConstructor = constructor;
        // TODO: we should manage the attributes and static parameters (even if it
        //       looks broken for the version 4.0)
        // class_definition.attributes = kJSClassAttributeNoAutomaticPrototype;
        // class_definition.staticValues = values;
        // class_definition.staticFunctions = log;

        // TODO: manage private datas if any, we give std::ptr::null_mut() for the
        //       moment.
        unsafe {
            let class = JSClassCreate([class_definition].as_ptr() as _);
            JSClassRetain(class);
            context.classes.push(class);
            JSObjectMake(context.get_ref(), class, std::ptr::null_mut()).into()
        }
    }
}

/// A JavaScript virtual machine.
#[derive(Clone)]
pub struct JSVirtualMachine {
    pub context_group: JSContextGroupRef,
    global_context: JSGlobalContextRef,
}

impl Drop for JSVirtualMachine {
    fn drop(&mut self) {
        unsafe {
            JSGarbageCollect(self.global_context);
            JSGlobalContextRelease(self.global_context);
            JSContextGroupRelease(self.context_group);
        }
    }
}

impl JSVirtualMachine {
    /// Creates a new `JSVirtualMachine` object from a context.
    fn from(context: JSContextRef) -> Self {
        let global_context = unsafe { JSContextGetGlobalContext(context) };
        unsafe {
            JSGlobalContextRetain(global_context);
        }
        let context_group = unsafe { JSContextGetGroup(global_context) };
        unsafe {
            JSContextGroupRetain(context_group);
        }
        Self {
            context_group,
            global_context,
        }
    }

    /// Creates a new `JSVirtualMachine` object.
    fn new() -> Self {
        let context_group = unsafe { JSContextGroupCreate() };
        let global_context =
            unsafe { JSGlobalContextCreateInGroup(context_group, std::ptr::null_mut()) };
        Self {
            context_group,
            global_context,
        }
    }
}

/// A JavaScript execution context.
pub struct JSContext {
    inner: JSContextRef,
    pub vm: JSVirtualMachine,
    exception: Option<JSValue>,
    classes: Vec<JSClassRef>,
}

impl Default for JSContext {
    fn default() -> Self {
        JSContext::new()
    }
}

impl From<JSContextRef> for JSContext {
    fn from(ctx: JSContextRef) -> Self {
        let vm = JSVirtualMachine::from(ctx);
        Self {
            inner: ctx,
            vm,
            exception: None,
            classes: vec![],
        }
    }
}
impl JSContext {
    /// Create a new context in the same virtual machine
    pub fn split(&self) -> Self {
        unsafe {
            let context = JSGlobalContextCreateInGroup(self.vm.context_group, std::ptr::null_mut());
            JSContextGroupRetain(self.vm.context_group);
            JSGlobalContextRetain(context);
            let mut vm = self.vm.clone();
            vm.global_context = context;
            Self {
                inner: context,
                vm,
                exception: self.exception.clone(),
                classes: vec![],
            }
        }
    }

    /// Get inner opaque object.
    pub fn get_ref(&self) -> JSContextRef {
        self.inner
    }

    /// Create a new `JSContext` object.
    ///
    /// Note that this associated function also creates a new `JSVirtualMachine`.
    /// If you want to create a `JSContext` object within an existing virtual
    /// machine, please use the `with_virtual_machine` associated function.
    pub fn new() -> Self {
        let vm = JSVirtualMachine::new();
        Self {
            inner: vm.global_context,
            vm,
            exception: None,
            classes: vec![],
        }
    }

    /// Create a new `JSContext` object within the provided `JSVirtualMachine`.
    pub fn with_virtual_machine(vm: JSVirtualMachine) -> Self {
        Self {
            inner: vm.global_context,
            vm,
            exception: None,
            classes: vec![],
        }
    }

    /// Returns the context global object.
    pub fn get_global_object(&self) -> JSObject {
        JSObject::from(unsafe { JSContextGetGlobalObject(self.inner) })
    }

    /// Return the exception thrown while evaluating a script.
    pub fn get_exception(&self) -> Option<&JSValue> {
        self.exception.as_ref()
    }

    /// Evaluate the script.
    ///
    /// Returns the value the script evaluates to. If the script throws an
    /// exception, this function returns `None`. You can query the thrown
    /// exception with the `get_exception` method.
    pub fn evaluate_script(&mut self, script: &str, starting_line_number: i32) -> Option<JSValue> {
        self.exception = None;
        let script = JSString::from_utf8(script.to_string()).unwrap();
        let this_object = std::ptr::null_mut();
        let source_url = std::ptr::null_mut();
        let mut exception: JSValueRef = std::ptr::null_mut();
        let value = unsafe {
            JSEvaluateScript(
                self.vm.global_context,
                script.inner,
                this_object,
                source_url,
                starting_line_number,
                &mut exception,
            )
        };
        let value = JSValue::from(value);
        if value.is_null(self) {
            self.exception = Some(JSValue::from(exception));
            None
        } else {
            Some(value)
        }
    }
}
