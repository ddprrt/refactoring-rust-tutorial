# Refactoring Rust

## Why do we want to refactor?

1. To make code more understandable, readable
2. To add our own Semantics on top of existing primitives
3. To reduce code duplication and thus potential errors
4. To make existing code more robust

## Refactoring Techniques

There are three levels:

1. Syntax, naming, semantics
   1. Embrace Rust specific Syntax, think in expressions
   2. Use the same naming conventions as in the rest of the standard library
2. Structs and Traits
   1. Create structs, implement common traits and conversion traits
   2. Use library-specific traits
   3. Use proper error handling
3. Design Patterns
   1. Depending on your case, use one of the available design patterns
   2. Rework around ownership

In this short session, we will focus on the second point: Struct and Traits. A LOT of refactoring can be done on this level. The first level is being well handled by Clippy. The third level is very use-case specific. 

## The Example

A simple key-value store

- `POST` to a specific `key` saves elements into a HashMap.

```rust

pub async fn post_kv(
    Path(key): Path<String>,
    TypedHeader(content_type): TypedHeader<ContentType>,
    State(state): State<SharedState>,
    data: Bytes,
) -> Result<String, ()> {
    state
        .write()
        .unwrap()
        .db
        .insert(key, (content_type.to_string(), data));
    Ok("OK".to_string())
}
```

- `GET` to the same `key` retrieves data or shows a 404

```rust
pub async fn get_kv(
    Path(key): Path<String>,
    State(state): State<SharedState>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    match state.read().unwrap().db.get(&key) {
        Some((content_type, data)) => Ok(([("content-type", content_type.clone())], data.clone())),
        None => Err((StatusCode::NOT_FOUND, "Key not found").into_response()),
    }
}
```

- Adding a `grayscale` filter at the end of a URL retrieves a stored image in grayscale format, otherwise it returns a `forbidden` error.

```rust

pub async fn grayscale(
    Path(key): Path<String>,
    State(state): State<SharedState>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let image = match state.read().unwrap().db.get(&key) {
        Some((content_type, data)) => {
            if content_type == "image/png" {
                image::load_from_memory(&data).unwrap()
            } else {
                return Err((
                    StatusCode::FORBIDDEN,
                    "Not possible to grayscale this type of image",
                )
                    .into_response());
            }
        }
        None => return Err((StatusCode::NOT_FOUND, "Key not found").into_response()),
    };

    let mut vec: Vec<u8> = Vec::new();

    let mut cursor = Cursor::new(&mut vec);
    image
        .grayscale()
        .write_to(&mut cursor, ImageOutputFormat::Png)
        .unwrap();
    let bytes: Bytes = vec.into();

    Ok(([("content-type", "image/png")], bytes).into_response())
}
```

## Things of notice

1. We use `Result` types in every handler
2. We `unwrap` all potentially problematic operations
3. We use the `IntoResponse` implementation for tuples to generate responses
   1. This is actually a pretty nifty feature of Axum.

## Step 1: Error Handling

We have `Result` return types and lots of `unwrap`s in our codebase. Goal is to

1. Make good use of the `Result`
2. Get rid of all `unwrap` --> they are a code smell and stuff might acutally break.

We are on Refactoring Level 2:

1. Introduce a new type (struct)
2. Apply traits to make it compatible with the ecosystem and language

The struct:

```rust
pub struct KVError(StatusCode, String);

impl KVError {
    pub fn new(status: StatusCode, message: impl ToString) -> Self {
        Self(status, message.to_string())
    }
}
```

Notice the `impl ToString`. We want to own a `String` struct, but we don't care where to get it from. It can be an owned `String`, but basically it can be anything that can be converted to a `String`.

Here we also have _idiomatic_ naming at hand.

- `ToString` gives us the `to_string` method. A `to_` prefix indicates an expensive operation. And this is true, the implementation to convert a `String` into another `String` via `to_string()` allocates new memory. 
- What you might want instead is `impl Into<String>`. Here, the implementation _might_ be expensive, but usually conversions can be cheap.

```rust
impl KVError {
    pub fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self(status, message.into())
    }
}
```

We can use this struct already. The error part of `Result` does not care about implementing the right traits. But we want to implement the `Error` trait anyways. Why? Because we make it compatible with the broader ecosystem. It might be that part of _Axum_ or any other part in your application just wants to have errors and doesn't care which one. Here you can use an Error trait object to catch all potential errors, effectively allowing for polymorphism.

Implement the `Error` trait and get two error messages: It needs to implement `Display` and `Debug` as well. Great, so we are able to log our errors. Nice.

```rust
#[derive(Debug)]
pub struct KVError(StatusCode, String);

impl KVError {
    pub fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self(status, message.into())
    }
}

impl std::fmt::Display for KVError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.0.as_str(), self.1)
    }
}

impl std::error::Error for KVError {}
```

To make the new Error compatible with Axum, we need to implement `IntoResponse`

```rust
impl IntoResponse for KVError {
    fn into_response(self) -> axum::response::Response {
        (self.0, self.1).into_response()
    }
}
```

This is actually what we did before, but now everything is nicely wrapped in a struct that allows us to add our own semantics to our code.

One more thing. All the `unwrap()` calls are related to a potential poison error in the `RwLock`. To make this easier to use, let's convert from a `PoisonError` to a `KVError`.

```rust
impl<T> From<PoisonError<T>> for KVError {
    fn from(_value: PoisonError<T>) -> Self {
        KVError::new(StatusCode::INTERNAL_SERVER_ERROR, "Error writing to DB")
    }
}
```

Now we can do the following.

1. Change all return types
2. Change `unwrap` to `?` --> Error propagation
3. Make shortcuts!

```rust
pub async fn post_kv(
    Path(key): Path<String>,
    TypedHeader(content_type): TypedHeader<ContentType>,
    State(state): State<SharedState>,
    data: Bytes,
) -> Result<String, KVError> {
    state
        .write()?
        .db
        .insert(key, (content_type.to_string(), data));
    Ok("OK".to_string())
}

pub async fn get_kv(
    Path(key): Path<String>,
    State(state): State<SharedState>,
) -> Result<impl IntoResponse, KVError> {
    match state.read()?.db.get(&key) {
        Some((content_type, data)) => Ok(([("content-type", content_type.clone())], data.clone())),
        None => Err(KVError::not_found()),
    }
}

pub async fn grayscale(
    Path(key): Path<String>,
    State(state): State<SharedState>,
) -> Result<impl IntoResponse, KVError> {
    let image = match state.read()?.db.get(&key) {
        Some((content_type, data)) => {
            if content_type == "image/png" {
                image::load_from_memory(&data).unwrap()
            } else {
                return Err(KVError::forbidden());
            }
        }
        None => return Err(KVError::not_found()),
    };

    let mut vec: Vec<u8> = Vec::new();

    let mut cursor = Cursor::new(&mut vec);
    image
        .grayscale()
        .write_to(&mut cursor, ImageOutputFormat::Png)
        .unwrap();
    let bytes: Bytes = vec.into();

    Ok(([("content-type", "image/png")], bytes).into_response())
}
```

Potential errors will be handled! Even better, we are going to give an error if a `POST` didn't work!