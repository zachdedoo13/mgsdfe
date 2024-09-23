use std::collections::HashMap;
use std::time::Duration;

use id_tree::{InsertBehavior, Node, NodeId, Tree};
use instant::Instant;

use crate::FunctionProfile;

/// basic first attempt
#[derive(Debug)]
pub struct PerformanceProfiler {
   pub active: bool,
   pub profiles: HashMap<String, FunctionProfile>,

   pub function_tree: Tree<String>,
   pub current_node: Option<NodeId>,

   pub stored_data_amount: u32,
   pub stored_cash_amount: u32,
   pub update_interval: Duration,

   last_dump: Instant,
}
impl Default for PerformanceProfiler {
   fn default() -> Self {
      Self {
         active: true,
         profiles: Default::default(),

         function_tree: Tree::new(),
         current_node: None,

         stored_data_amount: 100,
         stored_cash_amount: 10,
         update_interval: Duration::from_millis(250),
         last_dump: Instant::now(),
      }
   }
}

impl PerformanceProfiler {
   pub fn start_time_function<T: Into<String>>(&mut self, name: T) {
      if !self.active { return; }
      let name = name.into();

      match self.profiles.get_mut(name.as_str()) {
         None => {
            self.profiles.insert(name.clone(), FunctionProfile::default());
            self.start_time_function(name.clone());
         }
         Some(profile) => {
            profile.start();
         }
      }

      // node tree
      {
         match self.function_tree.root_node_id() {
            None => {
               let root = self.function_tree.insert(Node::new(name.clone()), InsertBehavior::AsRoot)
                   .expect(format!("Failed to inset root node {}", name).as_str());

               self.current_node = Some(root);
            }
            Some(_) => {
               let inserted_node = self.function_tree.insert(
                  Node::new(name.clone()),
                  InsertBehavior::UnderNode(self.current_node.as_ref()
                      .expect("Failed to grasp current node ")
                  )
               ).expect("Failed to insert node");

               self.current_node = Some(inserted_node);
            }
         }
      }
   }

   pub fn end_time_function<T: Into<String>>(&mut self, name: T) {
      if !self.active { return; }
      let name = name.into();

      match self.profiles.get_mut(name.as_str()) {
         None => {
            panic!("Timer ended without a start");
         }
         Some(profile) => {
            profile.end();
         }
      }

      // node tree
      {
         let current_id = self.current_node.as_ref().unwrap();
         let root_id = self.function_tree.root_node_id().expect("Failed to grab root");

         let root_node = self.function_tree.get(root_id).unwrap();
         let current_node = self.function_tree.get(current_id).unwrap();
         println!("\nRoot = {} Current = {}", root_node.data(), current_node.data());
         println!("IDS Root => {:?} Current => {:?}\n", current_id, root_id);

         if root_node.data() == current_node.data() {
            println!("Resetting tree");
            println!("\nTree => {:?}\n", self.function_tree);
            self.function_tree = Tree::new();
            self.current_node = None;
         }
         else {
            let current_node = self.function_tree.get(current_id).expect("Current node not in tree");
            let parent = current_node.parent().expect("Failed to get node parent").clone();

            println!("Recurring {}", current_node.data());

            self.current_node = Some(parent);
         }

      }
   }
}

impl PerformanceProfiler {
   pub fn resolve_profiler(&mut self) {
      if !self.active { return; }

      if self.last_dump.elapsed() > self.update_interval {
         self.last_dump = Instant::now();

         for (_, profile) in self.profiles.iter_mut() {
            profile.resolve(self.stored_cash_amount, self.stored_data_amount);
         }
      }
   }
}


