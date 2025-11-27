const { DetoxCircusEnvironment } = require('detox/runners/jest');

class DetoxEnvironment extends DetoxCircusEnvironment {
  constructor(config, context) {
    super(config, context);
  }

  async setup() {
    await super.setup();
  }

  async teardown() {
    await super.teardown();
  }
}

module.exports = DetoxEnvironment;
