use common::timer;

fn main() {
   let (one, two) = timer!({
      let test1 = {
         let mut str = String::new();

         for i in 0..5_000 {
            for j in 0..i {
               str.push_str(j.to_string().as_str());
            }
            str.push('\n');
         }

         str
      };

      let test2 = {
         let mut str = String::new();

         for i in 0..5_000 {
            for j in 0..i {
               str.push_str(j.to_string().as_str());
            }
            str.push('\n');
         }

         str
      };

      (test1, test2)
   });


   timer!("Time for check", {
      println!("Check -> {}", one == two);
   });
}