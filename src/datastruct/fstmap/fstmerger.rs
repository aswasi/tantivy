use std::collections::BinaryHeap;
use core::SegmentReader;
use super::FstMapStreamer;
use common::BinarySerializable;
use postings::TermInfo;
use std::cmp::Ordering;


pub struct HeapItem<'a, V> where V: 'a + BinarySerializable {
    pub streamer: FstMapStreamer<'a, V>,
    pub segment_ord: usize,
}

impl<'a, V> PartialEq for HeapItem<'a, V> where V: 'a + BinarySerializable {
    fn eq(&self, other: &Self) -> bool {
        self.segment_ord == other.segment_ord
    }
}

impl<'a, V> Eq for HeapItem<'a, V> where V: 'a + BinarySerializable {
}

impl<'a, V> PartialOrd for HeapItem<'a, V> where V: 'a + BinarySerializable {
    fn partial_cmp(&self, other: &HeapItem<'a, V>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a, V> Ord for HeapItem<'a, V> where V: 'a + BinarySerializable {
    fn cmp(&self, other: &HeapItem<'a, V>) -> Ordering {
        (&other.streamer.key(), &other.segment_ord).cmp(&(&self.streamer.key(), &self.segment_ord))
    }
}

/// Given a list of sorted term streams,
/// returns an iterator over sorted unique terms.
///
/// The item yield is actually a pair with
/// - the term
/// - a slice with the ordinal of the segments containing
/// the terms.
pub struct FstMerger<'a, V> where V: 'a + BinarySerializable {
    heap: BinaryHeap<HeapItem<'a, V>>,
    current_streamers: Vec<HeapItem<'a, V>>,
}

impl<'a, V> FstMerger<'a, V> where V: 'a + BinarySerializable {
    fn new(streams: Vec<FstMapStreamer<'a, V>>) -> FstMerger<'a, V> {
        FstMerger {
            heap: BinaryHeap::new(),
            current_streamers: streams
                .into_iter()
                .enumerate()
                .map(|(ord, streamer)| {
                    HeapItem {
                        streamer: streamer,
                        segment_ord: ord,
                    }
                })
                .collect(),
        }
    }

    fn advance_segments(&mut self) {
        let streamers = &mut self.current_streamers;
        let heap = &mut self.heap;
        for mut heap_item in streamers.drain(..) {
            if heap_item.streamer.advance() {
                heap.push(heap_item);
            }
        }
    }


    /// Advance the term iterator to the next term.
    /// Returns true if there is indeed another term
    /// False if there is none.
    pub fn advance(&mut self) -> bool {
        self.advance_segments();
        if let Some(head) = self.heap.pop() {
            self.current_streamers.push(head);
            loop {
                if let Some(next_streamer) = self.heap.peek() {
                    if self.current_streamers[0].streamer.key() != next_streamer.streamer.key() {
                        break;
                    } 
                }
                else { break; } // no more streamer.
                let next_heap_it = self.heap.pop().unwrap(); // safe : we peeked beforehand
                self.current_streamers.push(next_heap_it);
            }
            true
        } else {
            false
        }
    }
    
    /// Returns the current term.
    ///
    /// This method may be called
    /// iff advance() has been called before
    /// and "true" was returned.
    pub fn key(&self) -> &[u8] {
        &self.current_streamers[0].streamer.key()
    }

    /// Returns the sorted list of segment ordinals
    /// that include the current term.
    ///
    /// This method may be called
    /// iff advance() has been called before
    /// and "true" was returned.
    pub fn current_kvs(&self) -> &[HeapItem<'a, V>] {
        &self.current_streamers[..]
    }
}




    /*


    /// Returns the sorted list of segment ordinals
    /// that include the current term.
    ///
    /// This method may be called
    /// iff advance() has been called before
    /// and "true" was returned.
    pub fn segment_ords(&self) -> &[usize] {
        &self.current_segment_ords[..]
    }
    */  

/*
impl<'a, V> Streamer<'a> for FstMerger<'a, V> {
    type Item = &'a Term;

    fn next(&'a mut self) -> Option<Self::Item> {
        if self.advance() {
            Some(&self.current_term)
        } else {
            None
        }
    }
}
*/

impl<'a> From<&'a [SegmentReader]> for FstMerger<'a, TermInfo> where TermInfo: BinarySerializable {
    fn from(segment_readers: &'a [SegmentReader]) -> FstMerger<'a, TermInfo> {
        FstMerger::new(segment_readers
            .iter()
            .map(|reader| reader.term_infos().stream())
            .collect()  )
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use schema::{SchemaBuilder, Document, TEXT};
//     use core::Index;

//     #[test]
//     fn test_term_iterator() {
//         let mut schema_builder = SchemaBuilder::default();
//         let text_field = schema_builder.add_text_field("text", TEXT);
//         let index = Index::create_in_ram(schema_builder.build());
//         {
//             let mut index_writer = index.writer_with_num_threads(1, 40_000_000).unwrap();
//             {
//                 {
//                     let mut doc = Document::default();
//                     doc.add_text(text_field, "a b d f");
//                     index_writer.add_document(doc);
//                 }
//                 index_writer.commit().unwrap();
//             }
//             {
//                 {
//                     let mut doc = Document::default();
//                     doc.add_text(text_field, "a b c d f");
//                     index_writer.add_document(doc);
//                 }
//                 index_writer.commit().unwrap();
//             }
//             {
//                 {
//                     let mut doc = Document::default();
//                     doc.add_text(text_field, "e f");
//                     index_writer.add_document(doc);
//                 }
//                 index_writer.commit().unwrap();
//             }
//         }
//         index.load_searchers().unwrap();
//         let searcher = index.searcher();
//         let mut term_it = searcher.terms();
//         let mut terms = String::new();
//         while let Some(term) = term_it.next() {
//             terms.push_str(term.text());
//         }
//         assert_eq!(terms, "abcdef");
//     }

// }
