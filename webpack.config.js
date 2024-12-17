import path, { resolve } from 'path';
import HtmlWebpackPlugin from 'html-webpack-plugin';

export default {
  entry: path.resolve('src/index.js'),
  module: {
    rules: [
      {
        test: /\.s[ac]ss$/i, // Handle SASS and SCSS files
        use: [
          "style-loader", // Creates `style` nodes from JS strings
          "css-loader",   // Translates CSS into CommonJS
          "sass-loader"  // Compiles Sass to CSS
        ]
      },
      {
        test: /\.(png|svg|jpg|jpeg|gif|ico|webp)$/i,
        type: 'asset/resource'
      },
    ],
  },
  plugins: [
    new HtmlWebpackPlugin({
      template: './src/index.html',
      filename: 'index.html'
    })
  ],
  output: {
    path: path.resolve('dist'),
    clean: true,
    assetModuleFilename: 'images/[hash][ext][query]'
  }
};
