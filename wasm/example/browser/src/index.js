import React from 'react';
import ReactDOM from 'react-dom/client';
import './index.css';
import App from './App';
import CelParserComponent from "./CELParser";

const root = ReactDOM.createRoot(document.getElementById('root'));
root.render(
  <React.StrictMode>
    <CelParserComponent />
  </React.StrictMode>
);