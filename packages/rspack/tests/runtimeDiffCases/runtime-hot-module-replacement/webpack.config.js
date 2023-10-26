const webpack = require("webpack");
const hmr = new webpack.HotModuleReplacementPlugin();
hmr.apply = hmr.apply.bind(hmr);

module.exports = {
	plugins: [hmr]
};
