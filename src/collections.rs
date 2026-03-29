#[cfg(all(feature = "hashbrown-collections", feature = "std-collections"))]
compile_error!("enable only one of 'hashbrown-collections' or 'std-collections'");

#[cfg(not(any(feature = "hashbrown-collections", feature = "std-collections")))]
compile_error!("enable one of 'hashbrown-collections' or 'std-collections'");

#[cfg(feature = "hashbrown-collections")]
pub type Map<K, V> = hashbrown::HashMap<K, V>;

#[cfg(feature = "hashbrown-collections")]
pub type Set<T> = hashbrown::HashSet<T>;

#[cfg(feature = "std-collections")]
pub type Map<K, V> = std::collections::HashMap<K, V>;

#[cfg(feature = "std-collections")]
pub type Set<T> = std::collections::HashSet<T>;
