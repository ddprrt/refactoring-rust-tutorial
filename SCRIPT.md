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

Same for `ImageError`.

```rust
impl From<ImageError> for KVError {
    fn from(_value: ImageError) -> Self {
        KVError::new(StatusCode::BAD_REQUEST, "Error processing image")
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

## Step 2: Image Respone

Something that also bugs me is that we have primitive data types, tuples, arrays with tuples, etc. that we use to create a proper response. That's bogus. We expose details, but don't know anything about its semantics. Also why would we need to know that the first element of a tuple is an array with a tuple? This doesn't make any sense. Let's do the same procedure as before.

- Create our own type.

```rust
use hyper::body::Bytes;

struct ImageResponse(Bytes);

impl ImageResponse {
    fn new(bytes: impl Into<Bytes>) -> Self {
        Self(bytes.into())
    }
}
```

Tuple structs are amazing if you want to just have _your_ semantics on top of existing structs.

- Apply traits to make it compatible with the ecosystem.

```rust
impl IntoResponse for ImageResponse {
    fn into_response(self) -> axum::response::Response {
        ([("content-type", "image/png")], self.0).into_response()
    }
}
```

This might not look like much, but 

1. it puts a blanket to some nasty details.
2. we suddenly get a term that we can discuss about. It's an `ImageResponse`! We can do something with that.
3. It opens up new possibilites.

For example we see that we convert a `DynamicImage` into an array of bytes. Maybe we can use a proper conversion trait for that and add it to our `ImageResponse`?

Let's use `TryFrom` because as we see from the `unwrap`, this might fail.

```rust
impl TryFrom<&DynamicImage> for ImageResponse {
    type Error = KVError;

    fn try_from(value: &DynamicImage) -> Result<Self, Self::Error> {
        let mut vec: Vec<u8> = Vec::new();

        let mut cursor = Cursor::new(&mut vec);
        value
            .grayscale()
            .write_to(&mut cursor, ImageOutputFormat::Png)?;
        Ok(Self::new(vec))
    }
}
```

A few things we see happening because we already did some nice refactorings.

1. We can reuse `KVError` and use error propagation! This makes our code much easier
2. Since we use an `impl Into<Bytes>`, we actually just need to get something that is compatible and pass it as argument, fantastic.
3. We hide complexity. Arguable the most complex operation in this piece of code.


We deliberately used a borrowed `DynamicImage` to give u


```rust
impl TryFrom<DynamicImage> for ImageResponse {
    type Error = KVError;

    fn try_from(value: DynamicImage) -> Result<Self, Self::Error> {
        ImageResponse::try_from(&value)
    }
}
```

Even better. We can get rid of the first operation, the `grayscale` call, and move that to the outside. With that, we only encapsulate the conversion, not the operation. This makes our code really nice and tidy. We have two entities, types, that we can talk about: `KVError` and `ImageResponse`, we know what they can be created of, and we hide nasty internals.

The statement at the end now speaks volumens:

```rust
Ok(ImageResponse::try_from(image.grayscale()))
```

We create an `ImageResponse` from an `image` that we convert to `grayscale`. It's like the spoke word. No details are leaked, and in fact, they are free to change anytime. We found a single position for this to change, rather than it being scattered all across our codebase.

The drill was always the same.

1. Introduce a type, a struct, that carries our semantics
2. Apply traits to make it compatible with the ecosystem
3. Apply conversion traits to allow for easy From->Into conversions between types. 

## Step 3: The storage

The last bit that bugs me is the storage, there are some nuances in there that I don't quite like.

1. We only go for PNG images. 
2. We have the same tuple extraction as before. 
3. There's a lot of if/else branches that take up a lot of space for some very simple "is this key here, and is it an image"?

Let's go.

What helps us here is again, our own type that gives us more information on what we store, because currently, we store the `ContenType` Header and `Bytes`, and do a conversion once we hit a PNG. 

What we really want is to differentiate between an image and everything else, and an enum is a perfect types for that.

```rust
pub enum StoredType {
    Image(DynamicImage),
    Other(ContentType, Bytes),
}

impl StoredType {
    pub fn new(content_type: ContentType, bytes: Bytes) -> Result<Self, KVError> {
        if content_type.to_string().starts_with("image") {
            let image = image::load_from_memory(&bytes)?;
            Ok(StoredType::Image(image))
        } else {
            Ok(StoredType::Other(content_type, bytes))
        }
    }
}
```

This changes our semantics a bit. Before we just stored anything and retrieved it just like that. Now we store a `DynamicImage` type, losing the information on the original data and just sending PNG. 

Let's keep it like that for demo purposes, but keep in mind that this might need some rework if you e.g. intend to store and retrieve other image types as well. 

_Optional:_ Discuss the `Result` in `new`.

So we introduced the type, now let's implement the right ecosystem traits.

```rust
impl IntoResponse for &StoredType {
    fn into_response(self) -> axum::response::Response {
        match self {
            StoredType::Image(image) => match ImageResponse::try_from(image) {
                Ok(response) => response.into_response(),
                Err(image_error) => image_error.into_response(),
            },
            StoredType::Other(content_type, bytes) => {
                ([("content-type", content_type.to_string())], bytes.clone()).into_response()
            }
        }
    }
}
```

And after updating our state, we need to change all handler methods. But look at that. Every method basically becomes 2-3 lines of code.

```rust
pub async fn post_kv(
    Path(key): Path<String>,
    TypedHeader(content_type): TypedHeader<ContentType>,
    State(state): State<SharedState>,
    data: Bytes,
) -> Result<String, KVError> {
    let stored = StoredType::new(content_type, data)?;
    state.write()?.db.insert(key, stored);
    Ok("OK".to_string())
}

pub async fn get_kv(
    Path(key): Path<String>,
    State(state): State<SharedState>,
) -> Result<impl IntoResponse, KVError> {
    match state.read()?.db.get(&key) {
        Some(elem) => Ok(elem.into_response()),
        None => Err(KVError::not_found()),
    }
}

pub async fn grayscale(
    Path(key): Path<String>,
    State(state): State<SharedState>,
) -> Result<impl IntoResponse, KVError> {
    match state.read()?.db.get(&key) {
        Some(StoredType::Image(image)) => Ok(ImageResponse::try_from(image.grayscale())?),
        Some(StoredType::Other(_, _)) => Err(KVError::forbidden()),
        _ => Err(KVError::not_found()),
    }
}

```

And adding new handlers becomes just as easy.

```rust
pub async fn thumbnail(
    Path(key): Path<String>,
    State(state): State<SharedState>,
) -> Result<impl IntoResponse, KVError> {
    match state.read()?.db.get(&key) {
        Some(StoredType::Image(image)) => Ok(ImageResponse::try_from(image.resize(
            100,
            100,
            image::imageops::FilterType::Nearest,
        ))?),
        Some(StoredType::Other(_, _)) => Err(KVError::forbidden()),
        _ => Err(KVError::not_found()),
    }
}

```

The good thing is that we see all previous refactorings fall nicely into place:

1. `KVError` is used everywhere
2. We decided to store `DynamicImage`, which means that all other conversions align nicely
3. With proper error propagation we don't need to do a lot of _actual_ error handling, we just call `?`
4. Adding new features is easy
5. More sensible refactorings (preserving content type for images, etc.) happens on a type level, not on a handler level.
6. We introduced vocabulary and can easily grasp what is going on. 

## End

Questions.

My questions:
1. What should we do if we want to support multiple storages
2. How does it feel to have a `new` that returns a `Result`

