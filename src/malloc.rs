use core::{alloc::Layout, ptr::NonNull};

// pub trait Mallocator {
//     fn allocator(&self) -> &Self {
//         self
//     }

//     fn malloc(&self, layout: Layout) -> anyhow::Result<NonNull<[u8]>>;
//     fn remalloc(&self, ptr: NonNull<[u8]>, layout: Layout) -> anyhow::Result<NonNull<[u8]>>;

//     fn free(&self, ptr: NonNull<u8>);

//     #[inline]
//     fn talloc<T>(&self) -> anyhow::Result<NonNull<T>> {
//         self.malloc(Layout::new::<T>()).map(|x| x.cast::<T>())
//     }
//     #[inline]
//     fn talloc_array<T>(&self, count: usize) -> anyhow::Result<NonNull<[T]>> {
//         let layout = Layout::array::<T>(count)?;
//         self.malloc(layout)
//             .map(|x| NonNull::new(x.as_ptr() as *mut _).unwrap())
//     }

//     #[inline]
//     fn allocate_bytes(&self, size_bytes: usize) -> anyhow::Result<NonNull<[u8]>> {
//         let layout = Layout::array::<u8>(size_bytes)?;
//         self.malloc(layout)
//     }
// }
