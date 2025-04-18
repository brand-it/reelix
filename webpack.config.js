const path = require("path");
const HtmlWebpackPlugin = require("html-webpack-plugin");

module.exports = {
  entry: [path.resolve("src/setup.js"), path.resolve("src/index.js")],
  mode: process.env.TAURI_ENV_DEBUG === "true" ? "development" : "production",
  optimization: {
    splitChunks: {
      chunks: "all",
    },
  },
  module: {
    rules: [
      {
        test: /\.s[ac]ss$/i,
        use: ["style-loader", "css-loader", "sass-loader"],
      },
      {
        test: /\.(png|svg|jpg|jpeg|gif|ico|webp)$/i,
        type: "asset/resource",
        generator: {
          filename: "images/[name][ext]",
        },
      },
    ],
  },
  plugins: [
    new HtmlWebpackPlugin({
      template: "./src/index.html",
      filename: "index.html",
    }),
  ],
  resolve: {
    extensions: [".js", ".jsx"], // Auto-resolve .js files
    mainFiles: ["index"], // Automatically look for index.js
  },
  output: {
    path: path.resolve("dist"),
    clean: true,
    assetModuleFilename: "images/[hash][ext][query]",
  },
};
