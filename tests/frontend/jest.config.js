// Jest配置文件
// 配置前端集成测试环境

module.exports = {
  // 测试环境配置
  testEnvironment: 'jsdom',
  
  // 测试文件匹配模式
  testMatch: [
    '**/__tests__/**/*.(ts|tsx|js)',
    '**/*.(test|spec).(ts|tsx|js)'
  ],
  
  // 模块路径映射
  moduleNameMapping: {
    '^@/(.*)$': '<rootDir>/src/$1',
    '^@components/(.*)$': '<rootDir>/src/components/$1',
    '^@pages/(.*)$': '<rootDir>/src/pages/$1',
    '^@services/(.*)$': '<rootDir>/src/services/$1',
    '^@stores/(.*)$': '<rootDir>/src/stores/$1',
    '^@types/(.*)$': '<rootDir>/src/types/$1',
    '^@utils/(.*)$': '<rootDir>/src/utils/$1',
    '^@assets/(.*)$': '<rootDir>/src/assets/$1'
  },
  
  // 模块文件扩展名
  moduleFileExtensions: ['ts', 'tsx', 'js', 'jsx', 'json', 'node'],
  
  // 设置文件
  setupFilesAfterEnv: ['<rootDir>/tests/frontend/jest.setup.js'],
  
  // 转换配置
  transform: {
    '^.+\\.(ts|tsx)$': 'ts-jest',
  },
  
  // 覆盖配置
  collectCoverageFrom: [
    'src/**/*.{ts,tsx}',
    '!src/**/*.d.ts',
    '!src/**/*.stories.{ts,tsx}',
  ],
  
  // 覆盖率阈值
  coverageThreshold: {
    global: {
      branches: 70,
      functions: 70,
      lines: 70,
      statements: 70
    }
  },
  
  // 测试超时时间（毫秒）
  testTimeout: 30000,
  
  // 并发测试数
  maxWorkers: '50%',
  
  // 忽略的路径和文件
  testPathIgnorePatterns: [
    '/node_modules/',
    '/dist/',
    '/build/',
    '/coverage/'
  ],
  
  // 全局变量
  globals: {
    'ts-jest': {
      tsconfig: 'tsconfig.json'
    }
  }
};