export default {
  entry: './src/main.js',
  module: {
    rules: [
      {
        test: /\.s[ac]ss$/i,
        use: [
          "style-loader", // Creates `style` nodes from JS strings
          "css-loader",   // Translates CSS into CommonJS
          "sass-loader",  // Compiles Sass to CSS
        ],
      },
    ],
  },
};
