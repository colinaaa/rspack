use std::collections::HashMap;

use once_cell::sync::Lazy;
use rspack_core::{hmr::hmr_module, ResolvedURI};
use rspack_swc::{swc_atoms, swc_common, swc_ecma_parser, swc_ecma_transforms_base};
use swc_atoms::JsWord;
use swc_common::{chain, Mark};
use swc_ecma_parser::{EsConfig, Syntax};
use swc_ecma_transforms_base::resolver::resolver_with_mark;
use swc_ecma_transforms_testing::test;

fn syntax() -> Syntax {
  Syntax::Es(EsConfig {
    ..Default::default()
  })
}

static RESOLVED_IDS: Lazy<HashMap<JsWord, ResolvedURI>> = Lazy::new(|| {
  let mut resolved_ids: HashMap<JsWord, ResolvedURI> = Default::default();
  resolved_ids.insert(
    JsWord::from("./b"),
    ResolvedURI::new("/b.js".to_string(), false),
  );
  resolved_ids
});

// test!(
//   syntax(),
//   |_tester| {
//     let top_level_mark = Mark::fresh(Mark::root());

//     chain!(
//       resolver_with_mark(top_level_mark),
//       hmr_module("/a.js".to_string(), top_level_mark, &RESOLVED_IDS, false)
//     )
//   },
//   hmr_module_transform_basic,
//   r#"
//   import a from './b';
//   module.hot.accpet('./b', () => {});
//   export { a };
//   "#,
//   r#"
//   rs.define("/a.js", function(require, module, exports) {
//     "use strict";
//     function _interopRequireDefault(obj) {
//       return obj && obj.__esModule ? obj : {
//           default: obj
//       };
//     }
//     Object.defineProperty(exports, "__esModule", {
//         value: true
//     });
//     Object.defineProperty(exports, "a", {
//         enumerable: true,
//         get: function() {
//             return _b.default;
//         }
//     });
//     var _b = _interopRequireDefault(require("/b.js"));
//     module.hot.accpet("/b.js", ()=>{});
//   });
//   "#
// );

// test!(
//   syntax(),
//   |_tester| {
//     let top_level_mark = Mark::fresh(Mark::root());

//     chain!(
//       resolver_with_mark(top_level_mark),
//       hmr_module("/a.js".to_string(), top_level_mark, &RESOLVED_IDS, true)
//     )
//   },
//   hmr_module_transform_basic_for_entry,
//   r#"
//   import a from './b';
//   export { a };
//   "#,
//   r#"
//   rs.define("/a.js", function(require, module, exports) {
//     "use strict";
//     function _interopRequireDefault(obj) {
//       return obj && obj.__esModule ? obj : {
//           default: obj
//       };
//     }
//     Object.defineProperty(exports, "__esModule", {
//         value: true
//     });
//     Object.defineProperty(exports, "a", {
//         enumerable: true,
//         get: function() {
//             return _b.default;
//         }
//     });
//     var _b = _interopRequireDefault(require("/b.js"));
//   });
//   rs.require("/a.js");
//   "#
// );

// test!(
//   syntax(),
//   |_tester| {
//     let top_level_mark = Mark::fresh(Mark::root());

//     chain!(
//       resolver_with_mark(top_level_mark),
//       hmr_module("/a.js".to_string(), top_level_mark, &RESOLVED_IDS, false)
//     )
//   },
//   hmr_module_transform_require_inside_iife,
//   r#"
//   (function() {
//     const b = require('./b')
//     console.log(b);
//   })()
//   "#,
//   r#"
//   rs.define("/a.js", function(require, module, exports) {
//     "use strict";
//     (function() {
//         const b = require("/b.js");
//         console.log(b);
//     })();
//   });
//   "#
// );
