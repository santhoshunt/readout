import { mount } from 'svelte'
import 'katex/dist/katex.min.css'
import './lib/theme/base.css'
import './lib/theme/syntax.css'
import App from './App.svelte'

const app = mount(App, {
  target: document.getElementById('app')!,
})

export default app
