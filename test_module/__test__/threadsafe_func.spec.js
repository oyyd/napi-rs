const bindings = require('../index.node')

// console.log('bindings', bindings);

bindings.testThreadsafeFunction((...args) => {
  console.log('called', args)
})
