import { useState } from "react";
// import reactLogo from "./assets/react.svg";
// import viteLogo from "/vite.svg";
import "./App.css";

function App() {
  const [count, setCount] = useState(0);

  return (
    <div className="min-h-screen bg-gray-100">
      <div className="container mx-auto px-4 py-8">
        <div className="text-center">
          <h1 className="text-4xl font-bold text-gray-900 mb-8">
            AI Manager - 迁移版本
          </h1>
          <p className="text-lg text-gray-600 mb-8">
            从 Python/FastAPI 迁移到 Rust/Tauri
          </p>

          <div className="max-w-md mx-auto bg-white rounded-lg shadow-md p-6">
            <div className="flex justify-center space-x-4 mb-6">
              <div className="h-16 w-16 bg-blue-500 rounded-full animate-spin flex items-center justify-center text-white font-bold">V</div>
              <div className="h-16 w-16 bg-cyan-500 rounded-full animate-bounce flex items-center justify-center text-white font-bold">R</div>
            </div>

            <h2 className="text-2xl font-semibold text-center mb-4">开发环境测试</h2>

            <div className="text-center">
              <button
                className="bg-blue-500 hover:bg-blue-600 text-white font-medium py-2 px-4 rounded transition-colors"
                onClick={() => setCount((count) => count + 1)}
              >
                计数器: {count}
              </button>
            </div>

            <div className="mt-6 text-sm text-gray-500 text-center">
              <p>✅ React 正常工作</p>
              <p>✅ TypeScript 配置正确</p>
              <p>✅ Tailwind CSS 样式生效</p>
              <p>🔄 Tauri 后端待集成</p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

export default App;